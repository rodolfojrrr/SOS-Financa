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

base = Path('/mnt/data/SOS-Financa-V1-REVISADA/app')
storage_stub = r'''
(() => {
  let data = {
    profile:null, accounts:[], categories:[
      {id:'c-income',name:'Salário',kind:'income',icon:'wallet',parentId:null},
      {id:'c-expense',name:'Alimentação',kind:'expense',icon:'basket',parentId:null},
      {id:'c-debt',name:'Dívidas',kind:'expense',icon:'receipt',parentId:null}
    ], transactions:[], commitments:[], commitmentPayments:[], cards:[], cardPurchases:[], cardPayments:[], debts:[], debtPayments:[], budgets:[], transfers:[], goals:[]
  };
  let seq=0;
  const map={account:'accounts',category:'categories',transaction:'transactions',commitment:'commitments',commitment_payment:'commitmentPayments',card:'cards',card_purchase:'cardPurchases',card_payment:'cardPayments',debt:'debts',debt_payment:'debtPayments',budget:'budgets',transfer:'transfers',goal:'goals'};
  const clone=x=>JSON.parse(JSON.stringify(x));
  window.SOSStorage={
    isNative:()=>false,
    async getState(){return clone(data)},
    async saveEntity(type,payload){
      if(type==='profile'){data.profile={...payload};return '1'}
      const key=map[type],id=payload.id||`qa-${++seq}`,item={...payload,id};
      const idx=data[key].findIndex(x=>x.id===id);if(idx>=0)data[key][idx]=item;else data[key].push(item);return id;
    },
    async archiveEntity(type,id){const key=map[type];data[key]=data[key].filter(x=>x.id!==id)},
    async makeBackup(){return 'qa-backup'}, async getBackups(){return []}, async restoreBackup(){},
    async getDatabaseInfo(){return {path:'QA',size:0,counts:{}}}
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

    page.locator('button[data-route="manage"]').first.click()
    page.locator('button[data-manage-tab="catalogs"]').click()
    page.locator('button[data-action="account"]').click()
    page.locator('#modal-root input[name="name"]').fill('Conta principal')
    page.locator('#modal-root input[name="institution"]').fill('Banco teste')
    page.locator('#modal-root input[name="openingBalance"]').fill('1000,00')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Conta principal').first.is_visible(), 'cadastro de conta funciona pela interface')

    page.locator('button.fab').click()
    modal = page.locator('#modal-root .modal')
    ok(modal.is_visible(), 'botão de gasto rápido abre modal')
    page.locator('#modal-root input[name="amount"]').fill('4,50')
    page.locator('#modal-root input[name="description"]').fill('Bombom teste')
    page.locator('#modal-root select[name="accountId"]').select_option(index=1)
    page.locator('#modal-root select[name="categoryId"]').select_option(label='Alimentação')
    page.locator('#modal-root input[name="description"]').click()
    ok(modal.is_visible(), 'clicar dentro do modal não fecha a janela de cadastro')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    page.locator('button[data-manage-tab="transactions"]').click()
    ok(page.get_by_text('Bombom teste').count() >= 1, 'gasto rápido é salvo e aparece no gerenciamento')

    page.locator('button[data-manage-tab="cards"]').click()
    page.locator('button[data-action="card"]').click()
    page.locator('#modal-root input[name="name"]').fill('Cartão QA')
    page.locator('#modal-root input[name="bank"]').fill('Nubank')
    page.locator('#modal-root input[name="last4"]').fill('1234')
    page.locator('#modal-root input[name="limit"]').fill('5000,00')
    page.locator('#modal-root input[name="closeDay"]').fill('18')
    page.locator('#modal-root input[name="dueDay"]').fill('25')
    page.locator('#modal-root select[name="accountId"]').select_option(index=1)
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Cartão QA').first.is_visible(), 'cadastro de cartão funciona')

    page.locator('button[data-action="card-purchase"]').click()
    page.locator('#modal-root select[name="cardId"]').select_option(index=1)
    page.locator('#modal-root input[name="description"]').fill('Compra parcelada QA')
    page.locator('#modal-root input[name="total"]').fill('1200,00')
    page.locator('#modal-root input[name="purchaseDate"]').fill('2026-08-18')
    page.locator('#modal-root input[name="installments"]').fill('6')
    page.locator('#modal-root select[name="categoryId"]').select_option(label='Alimentação')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Compra parcelada QA').first.is_visible(), 'compra parcelada é salva pela interface')

    # Responsividade mobile sem recarregar o estado
    page.set_viewport_size({'width': 390, 'height': 844})
    page.wait_for_timeout(100)
    ok(page.locator('.mobile-nav').is_visible(), 'navegação mobile aparece em largura de celular')
    ok(not page.locator('.sidebar').is_visible(), 'barra lateral desktop some no celular')
    page.locator('button.mobile-item[data-route="more"]').click()
    ok(page.get_by_text('Mais opções').is_visible(), 'menu Mais funciona no celular')
    page.locator('button.fab').click()
    ok(page.locator('#modal-root .modal').is_visible(), 'gasto rápido continua acessível no celular')
    page.keyboard.press('Escape')
    overflow_390=page.evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1')
    ok(overflow_390, 'layout de 390 px não cria rolagem horizontal global')
    page.set_viewport_size({'width': 320, 'height': 700})
    page.wait_for_timeout(100)
    overflow_320=page.evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1')
    ok(overflow_320, 'layout de 320 px continua sem rolagem horizontal global')

    ok(not errors, f'interface roda sem erros de JavaScript/console: {errors}')
    browser.close()

print(f'\nTodos os {checks} testes de interface passaram.')
