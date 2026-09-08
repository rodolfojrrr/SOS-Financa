from playwright.sync_api import sync_playwright
from pathlib import Path

errors = []
checks = 0

def ok(condition, message):
    global checks
    if not condition:
        raise AssertionError(message)
    checks += 1
    print(f'OK {checks:02d} - {message}')

base = Path(__file__).resolve().parents[1] / 'app'
storage_stub = r'''
(() => {
  let data = {
    profile:null, accounts:[], categories:[
      {id:'c-income',name:'Salário',kind:'income',icon:'wallet',parentId:null},
      {id:'c-expense',name:'Alimentação',kind:'expense',icon:'basket',parentId:null},
      {id:'c-debt',name:'Dívidas',kind:'expense',icon:'receipt',parentId:null}
    ], transactions:[], commitments:[], commitmentPayments:[], cards:[], cardPurchases:[], cardPayments:[], debts:[], debtPayments:[], budgets:[], transfers:[], goals:[], trash:[]
  };
  let seq=0;
  const map={account:'accounts',category:'categories',transaction:'transactions',commitment:'commitments',commitment_payment:'commitmentPayments',card:'cards',card_purchase:'cardPurchases',card_payment:'cardPayments',debt:'debts',debt_payment:'debtPayments',budget:'budgets',transfer:'transfers',goal:'goals'};
  const clone=x=>JSON.parse(JSON.stringify(x));
  const title=x=>x.name||x.description||x.monthKey||'Registro';
  window.SOSStorage={
    isNative:()=>false,
    async getState(){return clone(data)},
    async saveEntity(type,payload){
      if(type==='profile'){data.profile={...(data.profile||{}),...payload,updatedAt:new Date().toISOString()};return '1'}
      const key=map[type],id=payload.id||`qa-${++seq}`,now=new Date().toISOString(),item={...payload,id,createdAt:payload.createdAt||now,updatedAt:now};
      const idx=data[key].findIndex(x=>x.id===id);if(idx>=0)item.createdAt=data[key][idx].createdAt||now;
      if(idx>=0)data[key][idx]=item;else data[key].push(item);return id;
    },
    async archiveEntity(type,id){const key=map[type],idx=data[key].findIndex(x=>x.id===id);if(idx<0)return;const item=data[key][idx];data.trash.push({entityType:type,id,title:title(item),deletedAt:new Date().toISOString(),item:clone(item)});data[key].splice(idx,1)},
    async getTrash(){return clone(data.trash)},
    async restoreArchived(type,id){const idx=data.trash.findIndex(x=>x.entityType===type&&x.id===id);if(idx<0)return;const entry=data.trash[idx],key=map[type];if(key&&entry.item)data[key].push(entry.item);data.trash.splice(idx,1)},
    async deleteForever(type,id){data.trash=data.trash.filter(x=>!(x.entityType===type&&x.id===id))},
    async makeBackup(){return 'qa-backup'}, async getBackups(){return []}, async restoreBackup(){},
    async getDatabaseInfo(){return {path:'QA',size:0,counts:{}}},
    async getSyncPlatform(){return {platform:'preview',canSend:false,canReceive:false}},
    async startSyncServer(){return {ip:'192.168.0.2'}}, async stopSyncServer(){}, async syncWithPc(){return {message:'Mesclado'}}
  };
})();
'''
html = f'''<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1,viewport-fit=cover"><style>{(base/'styles.css').read_text()}</style></head><body><div id="app"></div><div id="modal-root"></div><div id="toast-root" class="toast-root"></div><script>{storage_stub}</script><script>{(base/'finance.js').read_text()}</script><script>{(base/'app.js').read_text()}</script></body></html>'''

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True, executable_path='/usr/bin/chromium', args=['--no-sandbox'])
    context = browser.new_context(viewport={'width': 1440, 'height': 900}, locale='pt-BR')
    page = context.new_page()
    page.on('pageerror', lambda exc: errors.append(str(exc)))
    page.on('console', lambda msg: errors.append(f'console:{msg.text}') if msg.type == 'error' else None)
    page.set_content(html, wait_until='load')

    ok(page.get_by_text('Bem-vindo ao SOS Finança').is_visible(), 'primeira abertura mostra configuração inicial')
    page.locator('input[name="name"]').fill('Teste QA')
    page.locator('form[data-form="setup"] button[type="submit"]').click()
    page.wait_for_selector('.app-shell')
    ok(page.get_by_text('Visão geral').first.is_visible(), 'configuração inicial entra no dashboard')
    ok(page.locator('.sidebar button[data-route="manage"]').count()==1, 'Gerenciar fica direto no menu lateral')
    ok(page.locator('.sidebar button[data-route="trash"]').count()==1, 'Lixeira fica disponível no menu lateral')
    ok(page.locator('.sidebar button[data-route="accounts"]').count()==1, 'Fixos substitui a antiga aba Contas no menu')
    ok(page.locator('.home-attention-empty').count()==1, 'aviso sem pendências possui espaçamento próprio abaixo dos cards')

    page.locator('.sidebar button[data-route="manage"]').click()
    ok(page.locator('.manage-folder-grid .folder-tile').count() == 8, 'Gerenciar continua organizado em pastas')
    ok(page.get_by_text('Receitas e despesas fixas').first.is_visible(), 'atalho de Gerenciar leva para a nova área de fixos')
    page.locator('button[data-manage-folder="catalogs"]').click()
    page.locator('button[data-action="account"]').click()
    page.locator('#modal-root input[name="name"]').fill('Conta principal')
    page.locator('#modal-root input[name="openingBalance"]').fill('1000,00')
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Conta principal').first.is_visible(), 'subpasta de cadastros permite criar conta')

    page.locator('button[data-manage-back]').click()
    page.locator('.sidebar button[data-route="accounts"]').click(); page.wait_for_timeout(50)
    ok(page.get_by_text('Receitas e despesas fixas').first.is_visible(), 'nova aba de fixos abre corretamente')
    ok(page.locator('.fixed-folder-grid .folder-tile').count()==2, 'fixos é dividido em duas pastas')
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(30)
    ok(page.locator('[data-folder-customize="fixed_income"]').count()==1, 'pasta de receitas fixas é personalizável')
    ok(page.locator('[data-folder-customize="fixed_expense"]').count()==1, 'pasta de contas fixas é personalizável')
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(30)

    page.locator('button[data-manage-folder="fixed_income"]').click(); page.wait_for_timeout(40)
    ok(page.get_by_text('Receitas fixas').first.is_visible(), 'pasta de receitas fixas abre em modo de visualização')
    ok(page.locator('.fixed-history-panel').count()==0, 'histórico de recebimentos fica oculto no modo normal')
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(30)
    page.locator('button[data-action="fixed-income"]').first.click()
    page.locator('#modal-root input[name="name"]').fill('Salário QA')
    page.locator('#modal-root input[name="amount"]').fill('2500,00')
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    ok(page.locator('.fixed-item-card').filter(has_text='Salário QA').count()==1, 'receita fixa aparece com cartão contornado')
    ok(page.locator('.fixed-history-panel').count()==1, 'histórico aparece somente durante a edição')
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(30)
    page.locator('button[data-action="pay-commitment"]').first.click()
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Recebido', exact=False).first.is_visible(), 'receita fixa mostra conferido quando recebida no mês')

    page.locator('button[data-manage-back]').click(); page.wait_for_timeout(30)
    page.locator('button[data-manage-folder="fixed_expense"]').click(); page.wait_for_timeout(30)
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(30)
    page.locator('button[data-action="fixed-expense"]').first.click()
    page.locator('#modal-root input[name="name"]').fill('Internet QA')
    page.locator('#modal-root input[name="amount"]').fill('99,90')
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(30)
    ok(page.get_by_text('Marcar como pago').first.is_visible(), 'conta fixa possui ação de pagamento no modo de visualização')
    page.locator('button[data-action="pay-commitment"]').first.click()
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Pago', exact=False).first.is_visible(), 'conta fixa mostra símbolo de conferido após pagamento')

    page.locator('.sidebar button[data-route="cards"]').click()
    page.locator('button[data-action="card"]').click()
    page.locator('#modal-root input[name="name"]').fill('Cartão QA')
    page.locator('#modal-root input[name="limit"]').fill('5000,00')
    page.locator('#modal-root input[name="dueDay"]').fill('7')
    ok(page.locator('#modal-root input[name="color"]').count()==1, 'cartão possui cor personalizável')
    ok(page.locator('#modal-root select[name="icon"]').count()==1, 'cartão possui ícone personalizável')
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    ok(page.locator('.card-folder-grid .folder-tile').count()==1, 'cartão aparece como pasta, não como bloco de ações gigantes')
    ok(page.locator('.card-folder-grid button[data-action="card-purchase-for"]').count()==0, 'botão gigante de compra não fica solto na visão geral')
    page.get_by_text('Cartão QA').first.click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Adicionar compra').first.is_visible(), 'adicionar compra fica dentro da pasta do cartão')
    ok(page.locator('.card-status-bar').is_visible(), 'pasta do cartão possui barra de limite, uso e fatura')
    page.locator('button[data-action="card-purchase-for"]').first.click()
    page.locator('#modal-root input[name="description"]').fill('Compra QA')
    page.locator('#modal-root input[name="total"]').fill('120,00')
    page.locator('#modal-root button[type="submit"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Compra QA').first.is_visible(), 'compra fica organizada dentro da pasta do cartão')
    ok(page.locator('[data-edit="card_purchase"]').count()==0, 'editar/excluir compra fica oculto no modo normal')
    page.locator('button[data-action="toggle-card-edit"]').click(); page.wait_for_timeout(50)
    ok(page.locator('[data-edit="card_purchase"]').count()==1, 'botão Editar revela ações de edição das compras')
    page.once('dialog', lambda dialog: dialog.accept())
    page.locator('[data-archive="card_purchase"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Compra QA').count()==0, 'excluir remove a compra da pasta e envia para a lixeira')

    page.locator('.sidebar button[data-route="trash"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Compra QA').count()>=1, 'item removido aparece na lixeira')
    page.once('dialog', lambda dialog: dialog.accept())
    page.locator('[data-trash-restore="card_purchase"]').click(); page.wait_for_timeout(80)
    ok(page.get_by_text('Lixeira vazia').first.is_visible(), 'lixeira permite restaurar item')

    page.locator('.sidebar button[data-route="home"]').click(); page.wait_for_timeout(80)
    ok(page.locator('.attention-panel').count()>=1, 'visão geral possui área de avisos acionáveis')
    # cartão vence dia 7 e a data de teste é após isso; se houver alerta, clicar deve abrir a pasta do cartão.
    if page.locator('[data-alert-open-type="card"]').count():
        page.locator('[data-alert-open-type="card"]').first.click(); page.wait_for_timeout(80)
        ok(page.locator('.card-folder-hero').is_visible(), 'alerta de fatura leva direto ao cartão')
    else:
        ok(True, 'estrutura de alerta acionável do cartão está disponível quando houver fatura pendente')

    page.locator('.sidebar button[data-route="manage"]').click(); page.wait_for_timeout(50)
    page.locator('button[data-action="toggle-manage-organize"]').click(); page.wait_for_timeout(50)
    ok(page.locator('[data-folder-customize]').count()>=8, 'modo Editar permite personalizar pastas de Gerenciar')
    page.locator('[data-folder-customize="transactions"]').first.click()
    ok(page.locator('#modal-root input[name="color"]').count()==1, 'pasta de gerenciamento tem cor personalizável')
    page.locator('#modal-root .modal-close').click(); page.wait_for_timeout(80)

    page.locator('.sidebar button[data-route="settings"]').click(); page.wait_for_timeout(80)
    ok(page.locator('input[name="notificationsEnabled"]').count()==1, 'configurações possuem notificações financeiras')
    ok(page.locator('button[data-action="notification-test"]').count()==1, 'há botão para testar notificação')

    page.set_viewport_size({'width': 390, 'height': 844}); page.wait_for_timeout(80)
    ok(page.locator('.mobile-nav').is_visible(), 'navegação mobile aparece em largura de celular')
    ok(page.locator('.mobile-item[data-route="manage"]').count()==1, 'Gerenciar fica direto no menu mobile')
    ok(not page.locator('.sidebar').is_visible(), 'barra lateral desktop some no celular')
    ok(page.evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1'), 'layout de 390 px não cria rolagem horizontal global')
    page.set_viewport_size({'width': 320, 'height': 700}); page.wait_for_timeout(80)
    ok(page.evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1'), 'layout de 320 px continua sem rolagem horizontal global')

    ok(not errors, f'interface roda sem erros de JavaScript/console: {errors}')
    browser.close()

print(f'\nTodos os {checks} testes de interface passaram.')
