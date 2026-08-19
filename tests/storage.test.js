const mem = new Map();
global.window = {
  localStorage: {
    getItem: key => mem.has(key) ? mem.get(key) : null,
    setItem: (key, value) => mem.set(key, String(value)),
    removeItem: key => mem.delete(key)
  }
};
global.localStorage = window.localStorage;
global.crypto = require('crypto').webcrypto;
require('../app/storage.js');
const S = window.SOSStorage;

let passed = 0;
function expect(condition, message) {
  if (!condition) throw new Error(`FALHOU - ${message}`);
  passed += 1;
  console.log(`OK ${String(passed).padStart(2, '0')} - ${message}`);
}

(async () => {
  let state = await S.getState();
  expect(state.categories.length >= 10, 'prévia inicia com categorias úteis');

  await S.saveEntity('profile', { name: 'Teste', usageMode: 'household', theme: 'light' });
  const accountId = await S.saveEntity('account', { name: 'Conta principal', openingBalanceCents: 100000 });
  const txId = await S.saveEntity('transaction', { kind: 'expense', amountCents: 1000, description: 'Teste', date: '2026-08-18', accountId });
  state = await S.getState();
  expect(state.profile.name === 'Teste', 'salva perfil local');
  expect(state.accounts.some(x => x.id === accountId), 'salva conta local com UUID');
  expect(state.transactions.some(x => x.id === txId), 'salva lançamento local');

  let blocked = false;
  try { await S.archiveEntity('account', accountId); } catch (e) { blocked = /vinculad|possui/i.test(e.message); }
  expect(blocked, 'não deixa remover conta que possui histórico vinculado');

  await S.archiveEntity('transaction', txId);
  state = await S.getState();
  expect(!state.transactions.some(x => x.id === txId), 'estorno de lançamento remove efeito na prévia');

  await S.archiveEntity('account', accountId);
  state = await S.getState();
  expect(!state.accounts.some(x => x.id === accountId), 'conta sem vínculo pode ser removida na prévia');

  const debtId = await S.saveEntity('debt', { name: 'Dívida', balanceCents: 50000, originalCents: 50000, status: 'active' });
  blocked = false;
  try { await S.archiveEntity('debt', debtId); } catch (e) { blocked = /Quite|saldo/i.test(e.message); }
  expect(blocked, 'não deixa arquivar dívida com saldo em aberto');

  await S.saveEntity('debt_payment', { debtId, amountCents: 50000, principalCents: 50000, interestCents: 0, installmentMonth: '2026-08', date: '2026-08-18' });
  await S.archiveEntity('debt', debtId);
  state = await S.getState();
  expect(!state.debts.some(x => x.id === debtId), 'dívida quitada pode ser arquivada');

  console.log(`\nTodos os ${passed} testes de armazenamento da prévia passaram.`);
})().catch(err => { console.error(err); process.exit(1); });
