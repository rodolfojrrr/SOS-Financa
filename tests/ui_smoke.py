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
      const key=map[type],id=payload.id||`qa-${++seq}`,now=new Date().toISOString(),item={...payload,id,createdAt:payload.createdAt||now,updatedAt:now};
      const idx=data[key].findIndex(x=>x.id===id);if(idx>=0)item.createdAt=data[key][idx].createdAt||now;
      if(idx>=0)data[key][idx]=item;else data[key].push(item);return id;
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
    ok(page.locator('input[name="name"]').get_attribute('required') is None, 'nome do usuário também pode ser deixado em branco')
    page.locator('input[name="name"]').fill('Teste QA')
    page.locator('form[data-form="setup"] button[type="submit"]').click()
    page.wait_for_selector('.app-shell')
    ok(page.get_by_text('Visão geral').first.is_visible(), 'configuração inicial entra no dashboard')

    page.locator('button[data-route="manage"]').first.click()
    ok(page.get_by_text('Salário / receita fixa').first.is_visible(), 'gerenciamento mostra atalho explícito para salário fixo')
    page.locator('button[data-manage-tab="catalogs"]').click()
    page.locator('button[data-action="account"]').click()
    page.locator('#modal-root input[name="name"]').fill('Conta principal')
    page.locator('#modal-root input[name="openingBalance"]').fill('1000,00')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Conta principal').first.is_visible(), 'cadastro de conta funciona')

    page.locator('button[data-manage-tab="fixed_income"]').click()
    page.locator('button[data-action="fixed-income"]').first.click()
    page.locator('#modal-root input[name="name"]').fill('Salário QA')
    page.locator('#modal-root input[name="amount"]').fill('2500,00')
    ok(page.locator('#modal-root input[name="startMonth"]').get_attribute('required') is None, 'data inicial da receita fixa é opcional')
    ok(page.locator('#modal-root input[name="endMonth"]').get_attribute('required') is None, 'data final da receita fixa é opcional')
    ok(page.locator('#modal-root input[name="dueDay"]').get_attribute('required') is None, 'dia de recebimento da receita fixa é opcional')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Salário QA').first.is_visible(), 'salário fixo é salvo sem datas')
    page.locator('button[data-action="pay-commitment"]').first.click()
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Este mês registrado').first.is_visible(), 'recebimento do salário é registrado sem encerrar recorrência')
    salary_row=page.get_by_text('Salário QA').first.locator('xpath=ancestor::div[contains(@class,"list-row")]')
    salary_row.locator('button[data-edit="commitment"]').click()
    page.locator('#modal-root input[name="amount"]').fill('3000,00')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Este mês registrado').first.is_visible(), 'editar salário depois de receber não reabre o mês já quitado')
    page.locator('button[data-month-shift="1"]').first.click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Salário QA').first.is_visible(), 'salário continua existindo no mês seguinte')
    ok(page.get_by_text('R$ 3.000,00').count() >= 1, 'novo valor do salário passa a valer no mês seguinte')
    ok(page.get_by_text('Receber').first.is_visible(), 'mês seguinte volta a permitir receber salário')
    page.locator('button[data-month-shift="-1"]').first.click()

    page.locator('button[data-manage-tab="fixed_expense"]').click()
    page.locator('button[data-action="fixed-expense"]').first.click()
    page.locator('#modal-root input[name="name"]').fill('Internet QA')
    page.locator('#modal-root input[name="amount"]').fill('99,90')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Internet QA').first.is_visible(), 'conta fixa é salva sem data de início/fim')
    row=page.get_by_text('Internet QA').first.locator('xpath=ancestor::div[contains(@class,"list-row")]')
    row.locator('button[data-edit="commitment"]').click()
    page.locator('#modal-root input[name="amount"]').fill('109,90')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('R$ 109,90').count() >= 1, 'valor da conta fixa pode ser editado')

    page.locator('button.fab').click()
    page.locator('#modal-root input[name="amount"]').fill('4,50')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    page.locator('button[data-manage-tab="transactions"]').click()
    ok(page.get_by_text('Despesa').count() >= 1, 'gasto rápido aceita descrição vazia e usa texto padrão')

    page.locator('button[data-manage-tab="cards"]').click()
    page.locator('button[data-action="card"]').first.click()
    page.locator('#modal-root input[name="name"]').fill('Cartão QA')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Cartão QA').first.is_visible(), 'cartão pode ser cadastrado sem limite/fechamento/vencimento')
    page.locator('button[data-action="card-purchase-for"]').first.click()
    ok(page.locator('#modal-root select[name="cardId"]').input_value() != '', 'Adicionar compra dentro do cartão já deixa o cartão selecionado')
    page.locator('#modal-root input[name="total"]').fill('1200,00')
    page.locator('#modal-root input[name="description"]').fill('Compra parcelada QA')
    page.locator('#modal-root input[name="installments"]').fill('6')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    page.locator('button[data-card-open-route]').first.click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Compra parcelada QA').count() >= 1, 'tela do cartão mostra as compras daquele cartão')
    ok(page.get_by_text('Adicionar compra').first.is_visible(), 'tela do cartão possui botão próprio para adicionar compra')

    page.locator('button[data-route="manage"]').first.click()
    page.locator('button[data-manage-tab="debts"]').click()
    page.locator('button[data-action="debt"]').first.click()
    page.locator('#modal-root input[name="name"]').fill('Empréstimo antigo QA')
    page.locator('#modal-root input[name="installment"]').fill('450,00')
    ok(page.locator('#modal-root input[name="startDate"]').get_attribute('required') is None, 'data inicial da dívida é opcional')
    page.locator('#modal-root button[type="submit"]').click()
    page.wait_for_timeout(100)
    ok(page.get_by_text('Empréstimo antigo QA').first.is_visible(), 'dívida incompleta pode ser cadastrada')
    ok(page.get_by_text('saldo não informado').count() >= 1, 'dívida incompleta é identificada como saldo não informado')

    page.set_viewport_size({'width': 390, 'height': 844})
    page.wait_for_timeout(100)
    ok(page.locator('.mobile-nav').is_visible(), 'navegação mobile aparece em largura de celular')
    ok(not page.locator('.sidebar').is_visible(), 'barra lateral desktop some no celular')
    overflow_390=page.evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1')
    ok(overflow_390, 'layout de 390 px não cria rolagem horizontal global')
    page.set_viewport_size({'width': 320, 'height': 700})
    page.wait_for_timeout(100)
    overflow_320=page.evaluate('document.documentElement.scrollWidth <= window.innerWidth + 1')
    ok(overflow_320, 'layout de 320 px continua sem rolagem horizontal global')

    page.locator('.mobile-item[data-route="home"]').click()
    page.wait_for_timeout(50)
    page.locator('.mobile-item[data-route="cards"]').click()
    page.wait_for_timeout(50)
    page.go_back()
    page.wait_for_timeout(80)
    ok(page.get_by_text('Visão geral').first.is_visible(), 'botão Voltar do Android consegue retornar para a tela anterior do app')
    page.locator('button.fab').click()
    page.wait_for_timeout(50)
    ok(page.locator('#modal-root .modal').is_visible(), 'gasto rápido abre modal no mobile')
    page.go_back()
    page.wait_for_timeout(80)
    ok(page.locator('#modal-root .modal').count() == 0, 'botão Voltar do Android fecha o modal antes de sair do app')

    ok(not errors, f'interface roda sem erros de JavaScript/console: {errors}')
    browser.close()

print(f'\nTodos os {checks} testes de interface passaram.')
