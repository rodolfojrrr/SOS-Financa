process.env.TZ = 'America/Fortaleza';
global.window = {};
require('../app/finance.js');
const F = window.SOSFinance;

let passed = 0;
function expect(condition, message) {
  if (!condition) throw new Error(`FALHOU - ${message}`);
  passed += 1;
  console.log(`OK ${String(passed).padStart(2, '0')} - ${message}`);
}
function eq(actual, expected, message) {
  expect(actual === expected, `${message} | esperado=${expected} obtido=${actual}`);
}

const base = () => ({
  accounts: [], categories: [], transactions: [], commitments: [], commitmentPayments: [], cards: [],
  cardPurchases: [], cardPayments: [], debts: [], debtPayments: [], transfers: [], budgets: [], goals: []
});

// Datas e dinheiro
eq(F.todayKey(new Date('2026-08-18T02:30:00Z')), '2026-08-17', 'data local não pula para o dia seguinte por causa do UTC');
eq(F.centsFromInput('R$ 1.234,56'), 123456, 'converte valor brasileiro com milhar e vírgula');
eq(F.centsFromInput('1234.56'), 123456, 'converte valor decimal com ponto');
eq(F.clampDay('2026-02', 31), '2026-02-28', 'vencimento dia 31 é ajustado em fevereiro');
eq(F.clampDay('2028-02', 31), '2028-02-29', 'vencimento respeita ano bissexto');
eq(F.monthDistance('2025-12', '2026-02'), 2, 'distância entre meses atravessa virada do ano');
eq(F.monthsRange('2026-11', '2027-02').join(','), '2026-11,2026-12,2027-01,2027-02', 'intervalo de meses atravessa ano corretamente');

// Parcelamento e faturas
const parts = F.installmentParts(10000, 3);
eq(parts.reduce((a, x) => a + x, 0), 10000, 'parcelamento preserva exatamente o valor total');
eq(parts.join(','), '3334,3333,3333', 'centavos restantes são distribuídos sem perder dinheiro');

let st = base();
st.cards.push({ id: 'card-a', name: 'Nubank', limitCents: 500000, closeDay: 18, dueDay: 25 });
st.cardPurchases.push({ id: 'p-a', cardId: 'card-a', description: 'Compra', totalCents: 120000, purchaseDate: '2026-08-18', installments: 6 });
eq(F.invoiceFor(st, 'card-a', '2026-08').amountCents, 20000, 'compra no dia do fechamento entra na fatura esperada quando vence depois do fechamento');
eq(F.invoiceFor(st, 'card-a', '2027-01').amountCents, 20000, 'parcelamento ocupa a sexta fatura correta');
eq(F.invoiceFor(st, 'card-a', '2027-02').amountCents, 0, 'não cria parcela extra');

const cardB = { id: 'card-b', name: 'Cartão B', limitCents: 300000, closeDay: 25, dueDay: 5 };
st.cards.push(cardB);
eq(F.firstInvoiceMonth(cardB, '2026-08-20'), '2026-09', 'cartão que vence antes do fechamento joga compra anterior ao fechamento para o mês seguinte');
eq(F.firstInvoiceMonth(cardB, '2026-08-26'), '2026-10', 'compra depois do fechamento pula para a fatura subsequente');
eq(F.cardCommitted(st, 'card-a'), 120000, 'valor total parcelado compromete limite enquanto não pago');
st.cardPayments.push({ id: 'cp1', cardId: 'card-a', invoiceMonth: '2026-08', amountCents: 20000, date: '2026-08-25', accountId: null });
eq(F.cardCommitted(st, 'card-a'), 100000, 'pagamento de fatura libera limite');

// Conta bancária: cartão só baixa no pagamento da fatura
st = base();
st.accounts.push({ id: 'acc', openingBalanceCents: 250000 });
st.categories.push({ id: 'food', name: 'Alimentação', kind: 'expense' });
st.transactions.push({ id: 't1', kind: 'expense', amountCents: 450, date: '2026-08-18', accountId: 'acc', categoryId: 'food' });
st.cards.push({ id: 'card', name: 'Cartão', limitCents: 500000, closeDay: 18, dueDay: 25 });
st.cardPurchases.push({ id: 'p1', cardId: 'card', description: 'Compra', totalCents: 120000, purchaseDate: '2026-08-18', installments: 6, categoryId: 'food' });
st.debts.push({ id: 'debt', balanceCents: 1800000, installmentCents: 90000, startDate: '2026-08-01', installmentsPaid: 0, installmentsTotal: 48, dueDay: 10, status: 'active' });
st.debtPayments.push({ id: 'dp1', debtId: 'debt', amountCents: 90000, principalCents: 75000, interestCents: 15000, date: '2026-08-18', installmentMonth: '2026-08', accountId: 'acc' });
eq(F.debtBalance(st, st.debts[0]), 1725000, 'saldo devedor reduz somente pela amortização/principal');
eq(F.accountBalance(st, 'acc'), 159550, 'compra no cartão não baixa conta antes do pagamento da fatura');
st.cardPayments.push({ id: 'pay', cardId: 'card', invoiceMonth: '2026-08', amountCents: 20000, date: '2026-08-25', accountId: 'acc' });
eq(F.accountBalance(st, 'acc'), 139550, 'pagamento da fatura baixa a conta bancária');

// Dívidas antigas e parcelas históricas
st = base();
st.debts.push({ id: 'car', name: 'Carro', balanceCents: 1200000, installmentCents: 80000, startDate: '2025-01-10', installmentsPaid: 20, installmentsTotal: 48, dueDay: 10, status: 'active' });
expect(!F.debtDueInMonth(st.debts[0], '2026-08'), '20 parcelas históricas impedem agosto/2026 de aparecer falsamente em atraso');
expect(F.debtDueInMonth(st.debts[0], '2026-09'), 'primeira parcela ainda controlada começa depois das parcelas históricas');
eq(F.monthSummary(st, '2026-08').debtPlanned, 0, 'mês coberto pelas parcelas históricas não entra como dívida prevista');
eq(F.monthSummary(st, '2026-09').debtPlanned, 80000, 'primeira parcela nova entra no planejamento do mês correto');

// Pagamento atrasado com competência separada da data do pagamento
st.debts[0].installmentsPaid = 19;
st.debtPayments.push({ id: 'late', debtId: 'car', amountCents: 80000, principalCents: 70000, interestCents: 10000, date: '2026-09-02', installmentMonth: '2026-08' });
const augDebt = F.monthAgenda(st, '2026-08', '2026-09-03').find(x => x.type === 'debt');
expect(augDebt && augDebt.status === 'paid', 'pagamento feito em setembro consegue quitar a parcela referente a agosto');
eq(F.monthSummary(st, '2026-08').expensesRealized, 80000, 'competência da dívida reconhece parcela quitada no mês de referência');

// Contas recorrentes e atrasos atravessando mês
st = base();
st.commitments.push({ id: 'energy', name: 'Energia', kind: 'expense', amountCents: 18000, dueDay: 10, startMonth: '2026-07', active: true });
let buckets = F.dueBuckets(st, '2026-08', new Date('2026-08-18T12:00:00-03:00'));
expect(buckets.overdue.some(x => x.id === 'energy' && x.dueDate === '2026-07-10'), 'conta atrasada do mês anterior continua visível no mês atual');
expect(buckets.overdue.some(x => x.id === 'energy' && x.dueDate === '2026-08-10'), 'conta atrasada do mês atual também aparece');

st.commitments = [{ id: 'internet', name: 'Internet', kind: 'expense', amountCents: 10000, dueDay: 2, startMonth: '2026-09', active: true }];
buckets = F.dueBuckets(st, '2026-08', new Date('2026-08-29T12:00:00-03:00'));
expect(buckets.next7.some(x => x.dueDate === '2026-09-02'), 'próximos 7 dias enxergam vencimentos do mês seguinte');

// Recorrência com começo/fim
eq(F.commitmentActiveInMonth({ active: true, startMonth: '2026-03', endMonth: '2026-05' }, '2026-02'), false, 'recorrência não existe antes do início');
eq(F.commitmentActiveInMonth({ active: true, startMonth: '2026-03', endMonth: '2026-05' }, '2026-04'), true, 'recorrência existe dentro do período');
eq(F.commitmentActiveInMonth({ active: true, startMonth: '2026-03', endMonth: '2026-05' }, '2026-06'), false, 'recorrência termina no mês configurado');

console.log(`\nTodos os ${passed} testes financeiros passaram.`);
