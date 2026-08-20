(() => {
  const KEY = 'sos-financa-browser-v2';

  const defaults = () => ({
    profile: null,
    accounts: [],
    categories: [
      ['Salário', 'income', 'wallet'], ['Renda extra', 'income', 'plus'], ['Outras receitas', 'income', 'arrow-down'],
      ['Moradia', 'expense', 'home'], ['Alimentação', 'expense', 'basket'], ['Feira', 'expense', 'cart'],
      ['Transporte', 'expense', 'car'], ['Combustível', 'expense', 'fuel'], ['Saúde', 'expense', 'heart'],
      ['Educação', 'expense', 'book'], ['Lazer', 'expense', 'sparkles'], ['Assinaturas', 'expense', 'repeat'],
      ['Dívidas', 'expense', 'receipt'], ['Pequenos gastos', 'expense', 'coins'], ['Outros', 'expense', 'tag']
    ].map(([name, kind, icon], index) => ({ id: `browser-cat-${index + 1}`, name, kind, icon, parentId: null })),
    transactions: [], commitments: [], commitmentPayments: [], cards: [], cardPurchases: [], cardPayments: [],
    debts: [], debtPayments: [], budgets: [], transfers: [], goals: []
  });

  const entityMap = {
    account: 'accounts', category: 'categories', transaction: 'transactions', commitment: 'commitments',
    commitment_payment: 'commitmentPayments', card: 'cards', card_purchase: 'cardPurchases', card_payment: 'cardPayments',
    debt: 'debts', debt_payment: 'debtPayments', budget: 'budgets', transfer: 'transfers', goal: 'goals'
  };

  function hasTauri() {
    return Boolean(window.__TAURI__?.core?.invoke);
  }

  function readLocal() {
    try {
      const raw = localStorage.getItem(KEY);
      return raw ? { ...defaults(), ...JSON.parse(raw) } : defaults();
    } catch {
      return defaults();
    }
  }

  function writeLocal(data) {
    localStorage.setItem(KEY, JSON.stringify(data));
  }

  function stamp(obj, existing) {
    const now = new Date().toISOString();
    return { ...obj, id: obj.id || crypto.randomUUID(), createdAt: existing?.createdAt || now, updatedAt: now };
  }

  async function invoke(command, args = {}) {
    return window.__TAURI__.core.invoke(command, args);
  }

  window.SOSStorage = {
    isNative: hasTauri,
    async getState() {
      if (hasTauri()) return invoke('get_state');
      return readLocal();
    },
    async saveEntity(entityType, payload) {
      if (hasTauri()) return invoke('save_entity', { entityType, payload });
      const data = readLocal();
      if (entityType === 'profile') {
        data.profile = { ...payload, createdAt: data.profile?.createdAt || new Date().toISOString(), updatedAt: new Date().toISOString() };
        writeLocal(data);
        return '1';
      }
      const collection = entityMap[entityType];
      if (!collection) throw new Error('Tipo de registro não reconhecido');
      const list = data[collection] || [];
      const index = list.findIndex(item => item.id === payload.id);
      const item = stamp(payload, index >= 0 ? list[index] : null);
      if (index >= 0) list[index] = item; else list.push(item);
      data[collection] = list;
      writeLocal(data);
      return item.id;
    },
    async archiveEntity(entityType, id) {
      if (hasTauri()) return invoke('archive_entity', { entityType, id });
      const data = readLocal();
      const collection = entityMap[entityType];
      if (!collection) return;
      const has = (name, fn) => (data[name] || []).some(fn);
      if (entityType === 'account' && (
        has('transactions', x => x.accountId === id) || has('commitments', x => x.accountId === id) || has('cards', x => x.accountId === id) ||
        has('commitmentPayments', x => x.accountId === id) || has('cardPayments', x => x.accountId === id) || has('debtPayments', x => x.accountId === id) ||
        has('transfers', x => x.fromAccountId === id || x.toAccountId === id)
      )) throw new Error('Esta conta possui dados financeiros vinculados e não pode ser arquivada.');
      if (entityType === 'category' && (
        has('categories', x => x.parentId === id) || has('transactions', x => x.categoryId === id) || has('commitments', x => x.categoryId === id) ||
        has('cardPurchases', x => x.categoryId === id) || has('budgets', x => x.categoryId === id)
      )) throw new Error('Esta categoria possui dados vinculados e não pode ser arquivada.');
      if (entityType === 'commitment' && has('commitmentPayments', x => x.commitmentId === id)) throw new Error('Este valor recorrente possui histórico e não pode ser arquivado.');
      if (entityType === 'card' && (has('cardPurchases', x => x.cardId === id) || has('cardPayments', x => x.cardId === id))) throw new Error('Este cartão possui histórico e não pode ser arquivado.');
      if (entityType === 'debt') {
        const debt = (data.debts || []).find(x => x.id === id);
        const principal = (data.debtPayments || []).filter(x => x.debtId === id).reduce((a, x) => a + Number(x.principalCents || x.amountCents || 0), 0);
        if (debt && Math.max(0, Number(debt.balanceCents || 0) - principal) > 0) throw new Error('Quite ou ajuste o saldo desta dívida antes de arquivá-la.');
      }
      data[collection] = (data[collection] || []).filter(item => item.id !== id);
      writeLocal(data);
    },
    async makeBackup() {
      if (hasTauri()) return invoke('make_backup');
      const blob = new Blob([JSON.stringify(readLocal(), null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `sos_financa_preview_${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      return 'Backup JSON baixado no modo de prévia';
    },
    async getBackups() {
      if (hasTauri()) return invoke('get_backups');
      return [];
    },
    async restoreBackup(name) {
      if (hasTauri()) return invoke('restore_backup', { name });
      throw new Error('Restauração do banco SQLite só está disponível no aplicativo instalado.');
    },
    async getDatabaseInfo() {
      if (hasTauri()) return invoke('get_database_info');
      const raw = localStorage.getItem(KEY) || '';
      return { path: 'Prévia no navegador (localStorage)', size: new Blob([raw]).size, counts: {} };
    },
    async getSyncPlatform() {
      if (hasTauri()) return invoke('get_sync_platform');
      return { platform: 'preview', canSend: false, canReceive: false };
    },
    async startSyncServer() {
      if (hasTauri()) return invoke('start_sync_server');
      throw new Error('A sincronização Wi-Fi está disponível somente no aplicativo instalado.');
    },
    async stopSyncServer() {
      if (hasTauri()) return invoke('stop_sync_server');
    },
    async receiveSyncFromPc(host) {
      if (hasTauri()) return invoke('receive_sync_from_pc', { host });
      throw new Error('A sincronização Wi-Fi está disponível somente no aplicativo instalado.');
    },
    resetPreview() {
      if (!hasTauri()) localStorage.removeItem(KEY);
    }
  };
})();
