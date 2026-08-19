(() => {
  const MS_DAY = 86400000;
  const pad = n => String(n).padStart(2, '0');
  const todayKey = (date = new Date()) => `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  const monthKey = date => typeof date === 'string' ? date.slice(0, 7) : `${date.getFullYear()}-${pad(date.getMonth() + 1)}`;
  const parseDate = s => {
    if (!s) return new Date();
    const [y, m, d = '1'] = s.split('-').map(Number);
    return new Date(y, m - 1, d, 12, 0, 0);
  };
  const addMonths = (key, amount) => {
    const [y, m] = key.split('-').map(Number);
    const d = new Date(y, m - 1 + amount, 1, 12);
    return monthKey(d);
  };
  const monthDistance = (fromKey, toKey) => {
    const [fy, fm] = fromKey.split('-').map(Number);
    const [ty, tm] = toKey.split('-').map(Number);
    return (ty - fy) * 12 + (tm - fm);
  };
  const monthsRange = (fromKey, toKey) => {
    const distance = monthDistance(fromKey, toKey);
    if (!Number.isFinite(distance) || distance < 0) return [];
    return Array.from({ length: distance + 1 }, (_, i) => addMonths(fromKey, i));
  };
  const clampDay = (key, day) => {
    const [y, m] = key.split('-').map(Number);
    const max = new Date(y, m, 0).getDate();
    return `${key}-${pad(Math.min(Math.max(Number(day) || 1, 1), max))}`;
  };
  const money = cents => (Number(cents || 0) / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' });
  const centsFromInput = value => {
    if (typeof value === 'number') return Math.round(value * 100);
    let v = String(value || '').trim().replace(/\s/g, '').replace(/R\$/gi, '');
    if (!v) return 0;
    if (v.includes(',') && v.includes('.')) v = v.replace(/\./g, '').replace(',', '.');
    else if (v.includes(',')) v = v.replace(',', '.');
    const n = Number(v);
    return Number.isFinite(n) ? Math.round(n * 100) : 0;
  };
  const dateLabel = s => parseDate(s).toLocaleDateString('pt-BR');
  const monthLabel = key => parseDate(`${key}-01`).toLocaleDateString('pt-BR', { month: 'long', year: 'numeric' }).replace(/^./, c => c.toUpperCase());

  function byId(state, collection, id) {
    return (state[collection] || []).find(x => x.id === id);
  }
  function categoryName(state, id) { return byId(state, 'categories', id)?.name || 'Sem categoria'; }
  function accountName(state, id) { return byId(state, 'accounts', id)?.name || 'Sem conta'; }

  function installmentParts(totalCents, count) {
    count = Math.max(1, Number(count) || 1);
    const base = Math.floor(totalCents / count);
    const rem = totalCents - base * count;
    return Array.from({ length: count }, (_, i) => base + (i < rem ? 1 : 0));
  }

  function firstInvoiceMonth(card, purchaseDate) {
    const date = parseDate(purchaseDate);
    const key = monthKey(date);
    const day = date.getDate();
    const close = Number(card.closeDay || 1);
    const due = Number(card.dueDay || 10);
    if (due > close) return day <= close ? key : addMonths(key, 1);
    return day <= close ? addMonths(key, 1) : addMonths(key, 2);
  }

  function purchaseSchedule(state, purchase) {
    const card = byId(state, 'cards', purchase.cardId);
    if (!card) return [];
    const first = firstInvoiceMonth(card, purchase.purchaseDate);
    return installmentParts(Number(purchase.totalCents || 0), Number(purchase.installments || 1)).map((amountCents, index) => ({
      index: index + 1,
      monthKey: addMonths(first, index),
      amountCents,
      dueDate: clampDay(addMonths(first, index), card.dueDay),
      cardId: card.id,
      purchaseId: purchase.id,
      categoryId: purchase.categoryId,
      description: purchase.description,
      totalInstallments: Number(purchase.installments || 1)
    }));
  }

  function invoiceFor(state, cardId, key) {
    const items = (state.cardPurchases || []).filter(p => p.cardId === cardId).flatMap(p => purchaseSchedule(state, p)).filter(x => x.monthKey === key);
    const amountCents = items.reduce((a, x) => a + x.amountCents, 0);
    const paidCents = (state.cardPayments || []).filter(p => p.cardId === cardId && p.invoiceMonth === key).reduce((a, p) => a + Number(p.amountCents || 0), 0);
    return { items, amountCents, paidCents, remainingCents: Math.max(0, amountCents - paidCents) };
  }

  function cardCommitted(state, cardId) {
    const purchases = (state.cardPurchases || []).filter(p => p.cardId === cardId).reduce((sum, p) => sum + Number(p.totalCents || 0), 0);
    const payments = (state.cardPayments || []).filter(p => p.cardId === cardId).reduce((sum, p) => sum + Number(p.amountCents || 0), 0);
    return Math.max(0, purchases - payments);
  }

  function debtBalance(state, debt) {
    const reduced = (state.debtPayments || []).filter(p => p.debtId === debt.id).reduce((sum, p) => sum + Number(p.principalCents || p.amountCents || 0), 0);
    return Math.max(0, Number(debt.balanceCents || debt.totalContractCents || debt.originalCents || 0) - reduced);
  }

  function debtPaidCount(state, debt) {
    const historical = Math.max(0, Number(debt.installmentsPaid || 0));
    const registered = (state.debtPayments || []).filter(p => p.debtId === debt.id).length;
    return historical + registered;
  }

  function debtMonthIndex(debt, key) {
    if (!debt?.startDate) return 0;
    return monthDistance(monthKey(debt.startDate), key);
  }

  function debtDueInMonth(debt, key) {
    const index = debtMonthIndex(debt, key);
    if (index < 0) return false;
    const historical = Math.max(0, Number(debt.installmentsPaid || 0));
    if (index < historical) return false;
    const total = Math.max(0, Number(debt.installmentsTotal || 0));
    if (total > 0 && index >= total) return false;
    return Number(debt.installmentCents || 0) > 0;
  }

  function debtPaymentMonth(payment) {
    return payment.installmentMonth || monthKey(payment.date);
  }

  function commitmentActiveInMonth(c, key) {
    if (!c.active) return false;
    if (c.startMonth && key < c.startMonth) return false;
    if (c.endMonth && key > c.endMonth) return false;
    return true;
  }

  function commitmentPaid(state, c, key) {
    return (state.commitmentPayments || []).filter(p => p.commitmentId === c.id && p.monthKey === key).reduce((sum, p) => sum + Number(p.amountCents || 0), 0);
  }

  function accountBalance(state, accountId) {
    const account = byId(state, 'accounts', accountId);
    if (!account) return 0;
    let total = Number(account.openingBalanceCents || 0);
    for (const t of state.transactions || []) if (t.accountId === accountId) total += t.kind === 'income' ? Number(t.amountCents || 0) : -Number(t.amountCents || 0);
    for (const p of state.commitmentPayments || []) {
      if (p.accountId !== accountId) continue;
      const c = byId(state, 'commitments', p.commitmentId);
      if (c) total += c.kind === 'income' ? Number(p.amountCents || 0) : -Number(p.amountCents || 0);
    }
    for (const p of state.cardPayments || []) if (p.accountId === accountId) total -= Number(p.amountCents || 0);
    for (const p of state.debtPayments || []) if (p.accountId === accountId) total -= Number(p.amountCents || 0);
    for (const t of state.transfers || []) {
      if (t.fromAccountId === accountId) total -= Number(t.amountCents || 0);
      if (t.toAccountId === accountId) total += Number(t.amountCents || 0);
    }
    return total;
  }

  function currentBalance(state) {
    return (state.accounts || []).reduce((sum, a) => sum + accountBalance(state, a.id), 0);
  }

  function monthTransactions(state, key) {
    return (state.transactions || []).filter(t => monthKey(t.date) === key);
  }

  function monthSummary(state, key) {
    const txs = monthTransactions(state, key);
    const directIncome = txs.filter(t => t.kind === 'income').reduce((a, t) => a + Number(t.amountCents || 0), 0);
    const directExpense = txs.filter(t => t.kind === 'expense').reduce((a, t) => a + Number(t.amountCents || 0), 0);

    const commitments = (state.commitments || []).filter(c => commitmentActiveInMonth(c, key));
    let recurringIncome = 0, recurringExpense = 0, recurringIncomePaid = 0, recurringExpensePaid = 0;
    commitments.forEach(c => {
      const amount = Number(c.amountCents || 0);
      const paid = commitmentPaid(state, c, key);
      if (c.kind === 'income') { recurringIncome += amount; recurringIncomePaid += Math.min(amount, paid); }
      else { recurringExpense += amount; recurringExpensePaid += Math.min(amount, paid); }
    });

    const cardInvoices = (state.cards || []).map(card => ({ card, ...invoiceFor(state, card.id, key) }));
    const cardPlanned = cardInvoices.reduce((a, x) => a + x.amountCents, 0);
    const cardPaid = cardInvoices.reduce((a, x) => a + x.paidCents, 0);

    const debtPayments = (state.debtPayments || []).filter(p => debtPaymentMonth(p) === key);
    const debtPaid = debtPayments.reduce((a, p) => a + Number(p.amountCents || 0), 0);
    const debtPlanned = (state.debts || []).filter(d => d.status !== 'paid' && debtBalance(state, d) > 0 && debtDueInMonth(d, key)).reduce((a, d) => a + Number(d.installmentCents || 0), 0);

    const incomePlanned = directIncome + recurringIncome;
    const expensesPlanned = directExpense + recurringExpense + cardPlanned + debtPlanned;
    const incomeRealized = directIncome + recurringIncomePaid;
    const expensesRealized = directExpense + recurringExpensePaid + cardPaid + debtPaid;

    return {
      directIncome, directExpense, recurringIncome, recurringExpense, cardPlanned, debtPlanned,
      incomePlanned, expensesPlanned, projected: incomePlanned - expensesPlanned,
      incomeRealized, expensesRealized, realized: incomeRealized - expensesRealized,
      commitments, cardInvoices
    };
  }

  function expenseByCategory(state, key) {
    const map = new Map();
    const add = (id, amount) => map.set(id || 'none', (map.get(id || 'none') || 0) + Number(amount || 0));
    monthTransactions(state, key).filter(t => t.kind === 'expense').forEach(t => add(t.categoryId, t.amountCents));
    (state.commitmentPayments || []).filter(p => p.monthKey === key).forEach(p => {
      const c = byId(state, 'commitments', p.commitmentId); if (c?.kind === 'expense') add(c.categoryId, p.amountCents);
    });
    (state.cardPurchases || []).forEach(p => purchaseSchedule(state, p).filter(x => x.monthKey === key).forEach(x => add(p.categoryId, x.amountCents)));
    const debtCategory = (state.categories || []).find(c => c.kind === 'expense' && c.name.toLowerCase().includes('dívid'));
    (state.debtPayments || []).filter(p => monthKey(p.date) === key).forEach(p => add(debtCategory?.id, p.amountCents));
    return [...map.entries()].map(([categoryId, amountCents]) => ({ categoryId, amountCents })).sort((a, b) => b.amountCents - a.amountCents);
  }

  function monthAgenda(state, key, today = todayKey()) {
    const items = [];
    (state.commitments || []).filter(c => commitmentActiveInMonth(c, key)).forEach(c => {
      const dueDate = clampDay(key, c.dueDay);
      const paid = commitmentPaid(state, c, key);
      items.push({ type: 'commitment', id: c.id, label: c.name, dueDate, amountCents: Number(c.amountCents || 0), paidCents: paid, kind: c.kind, status: paid >= Number(c.amountCents || 0) ? 'paid' : dueDate < today ? 'overdue' : 'pending' });
    });
    (state.cards || []).forEach(card => {
      const inv = invoiceFor(state, card.id, key);
      if (inv.amountCents > 0) {
        const dueDate = clampDay(key, card.dueDay);
        items.push({ type: 'card', id: card.id, label: `Fatura ${card.name}`, dueDate, amountCents: inv.amountCents, paidCents: inv.paidCents, kind: 'expense', status: inv.remainingCents <= 0 ? 'paid' : dueDate < today ? 'overdue' : 'pending' });
      }
    });
    (state.debts || []).filter(d => d.status !== 'paid' && debtBalance(state, d) > 0 && debtDueInMonth(d, key)).forEach(d => {
      const dueDate = clampDay(key, d.dueDay);
      const paidThisMonth = (state.debtPayments || []).filter(p => p.debtId === d.id && debtPaymentMonth(p) === key).reduce((a, p) => a + Number(p.amountCents || 0), 0);
      items.push({ type: 'debt', id: d.id, label: d.name, dueDate, amountCents: Number(d.installmentCents || 0), paidCents: paidThisMonth, kind: 'expense', status: paidThisMonth >= Number(d.installmentCents || 0) ? 'paid' : dueDate < today ? 'overdue' : 'pending' });
    });
    return items.sort((a, b) => a.dueDate.localeCompare(b.dueDate));
  }

  function dueBuckets(state, key = monthKey(new Date()), now = new Date()) {
    const todayText = todayKey(now);
    const today = parseDate(todayText);
    const in7 = new Date(today.getTime() + 7 * MS_DAY);
    const currentKey = monthKey(today);
    const candidates = [currentKey];

    (state.commitments || []).forEach(c => {
      if (c.active && c.startMonth) candidates.push(c.startMonth);
    });
    (state.debts || []).forEach(d => {
      if (d.startDate) candidates.push(addMonths(monthKey(d.startDate), Math.max(0, Number(d.installmentsPaid || 0))));
    });
    (state.cardPurchases || []).forEach(p => {
      const schedule = purchaseSchedule(state, p);
      if (schedule.length) candidates.push(schedule[0].monthKey);
    });

    let earliest = candidates.sort()[0] || currentKey;
    const maxLookback = addMonths(currentKey, -36);
    if (earliest < maxLookback) earliest = maxLookback;
    const through = addMonths(currentKey, 1);
    const result = { overdue: [], next7: [], later: [] };

    monthsRange(earliest, through).flatMap(month => monthAgenda(state, month, todayText))
      .filter(x => x.kind === 'expense' && x.status !== 'paid')
      .forEach(item => {
        const d = parseDate(item.dueDate);
        if (d < today) result.overdue.push(item);
        else if (d <= in7) result.next7.push(item);
        else if (monthKey(item.dueDate) === key) result.later.push(item);
      });

    return result;
  }

  function lastMonths(key, count = 6) {
    return Array.from({ length: count }, (_, i) => addMonths(key, i - count + 1));
  }

  window.SOSFinance = {
    pad, todayKey, monthKey, addMonths, monthDistance, monthsRange, clampDay, money, centsFromInput, dateLabel, monthLabel, parseDate,
    byId, categoryName, accountName, installmentParts, firstInvoiceMonth, purchaseSchedule, invoiceFor, cardCommitted,
    debtBalance, debtPaidCount, debtMonthIndex, debtDueInMonth, debtPaymentMonth, commitmentActiveInMonth, commitmentPaid, accountBalance, currentBalance, monthTransactions,
    monthSummary, expenseByCategory, monthAgenda, dueBuckets, lastMonths
  };
})();
