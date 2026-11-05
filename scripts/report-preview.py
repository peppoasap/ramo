#!/usr/bin/env python3
"""Build an offline Italian report from actual captured benchmark JSON."""
from pathlib import Path
import json, statistics, html
ROOT=Path(__file__).resolve().parents[1]
def read(name):return json.loads((ROOT/'docs/verification'/name).read_text())
def main():
    data=read('delivery-performance.json'); machine=read('delivery-environment.json')
    footprint=read('distribution-footprint.json'); render=read('delivery-history-render.json')
    checks=read('preview-validation.json')
    assert data['profile']=='release'
    labels={
      'registerWarmRepositoryMs':'Aprire/registrare il repository già in cache',
      'cleanSnapshotMs':'Aggiornare stato e riferimenti, repository pulito',
      'dirtySnapshot100Ms':'Aggiornare stato con 100 file modificati',
      'history300Ms':'Leggere una pagina di 300 commit',
      'historyLastPageMs':'Leggere ultima pagina (200 commit, offset 4.800)',
      'commitFilesMs':'Elencare i file di un commit',
      'commitDiffMs':'Leggere il diff testuale di un commit',
      'workingDiffMs':'Leggere il diff di un file modificato',
      'stage100FilesMs':'Stage di 100 file',
      'unstage100FilesMs':'Unstage di 100 file',
      'commitOneFileMs':'Commit di un file, lasciando gli altri 99 fuori',
      'switchBranchMs':'Cambiare branch con lo stesso albero di file',
      'mergeThreeWayMs':'Merge locale divergente senza conflitti',
    }
    assert set(labels)==set(data['samples'])
    rows=[]
    for key,label in labels.items():
      values=data['samples'][key]
      assert len(values)==9 and all(isinstance(v,(int,float)) and v>0 for v in values)
      rows.append((label,statistics.median(values),min(values),max(values)))
    ramo_mb=footprint['ramo']['logicalFileBytes']/1e6
    kraken_mb=footprint['gitkraken']['logicalFileBytes']/1e6
    reduction=100*(1-ramo_mb/kraken_mb)
    intro=f'''# Ramo 0.1.0 — report di test e prestazioni

4 novembre 2026 · Prima preview privata per macOS Apple Silicon.

La preview passa **{checks['frontendTests']} test frontend e {checks['backendTests']} backend**. Su questo fixture le operazioni Git interne misurate hanno mediane sotto mezzo secondo. Questo dato non comprende il tempo necessario per rivedere o confermare una modifica nell'interfaccia.

**Non abbiamo ancora una percentuale attendibile di vantaggio di velocità su GitKraken.** GitKraken 12.6.1 è installato e ha caricato correttamente la stessa cronologia sintetica di 5.011 commit usata nell'accettazione nativa di Ramo. Mancano misure end-to-end equivalenti: confrontare il backend di Ramo con un clic/una lettura automatizzata dell'interfaccia di GitKraken sarebbe scorretto. Nessun rapporto “X volte più veloce” viene attribuito a queste prove.

## Condizioni e metodo

- Mac con {machine['cpu']}, {machine['memoryBytes']//(1024**3)} GiB RAM, macOS {machine['system']}, architettura {machine['architecture']}.
- Ramo 0.1.0, build Rust **release**, {data['git']}; Node {render['node']} per la sola prova di rendering statico.
- Repository reale, sintetico e temporaneo: **5.000 commit lineari e 2.000 piccoli file testuali**. Fase dirty: 100 file modificati. Fase merge separata: due commit divergenti e due file aggiuntivi, con un riferimento diverso per ogni campione.
- Ogni operazione ha un primo campione scartato per riscaldamento, poi **9 campioni conservati**. Nessuna eliminazione degli outlier nei campioni riportati. Preparazione/reset del fixture e compilazione sono fuori dagli intervalli.
- Cronometro intorno alla vera chiamata Backend: sottoprocessi Git e, per le scritture, controllo di freschezza e snapshot del risultato sono inclusi. IPC, UI, revisione umana, rete e inferenza AI sono esclusi.
- “Aprire” indica registrazione su repository/file in cache, **non avvio a freddo dell'app**. Il cambio branch ha lo stesso albero: non misura checkout massivi. Stage/unstage sono controllati; il commit deve lasciare 99 file fuori, il merge deve produrre due genitori e un albero pulito con entrambi i file.
- Misura locale su macchina utilizzata anche per altre attività, senza isolamento CPU, controllo energetico o svuotamento cache. Sono valori osservati, non garanzie o stime rappresentative di tutta la popolazione.

## Tempi delle operazioni Ramo

Millisecondi, valori più bassi migliori. Il massimo è il massimo osservato su nove prove, non una stima affidabile del p95.

| Operazione | Mediana ms | Minimo ms | Massimo ms |
| --- | ---: | ---: | ---: |
'''
    table=''.join(f'| {label} | {median:.2f} | {low:.2f} | {high:.2f} |\n' for label,median,low,high in rows)
    render_table='\n'.join(f"| {r['loadedCommits']:,} | {r['renderedRows']} | {r['medianStaticRenderMs']:.2f} |" for r in render['measurements'])
    rest=f'''
## Grafo: carico di rendering

Prova separata del vero componente History tramite rendering statico server in Node, viewport iniziale 600px, sette campioni dopo riscaldamento. Include il layout dell'intera topologia caricata, ma **non misura frame rate, scroll nativo, memoria o tempo da clic a pixel**. I dati di questo test sono array sintetici; quelli del backend sono veri oggetti Git.

| Commit caricati | Righe renderizzate inizialmente | Mediana rendering statico ms |
| ---: | ---: | ---: |
{render_table}

La virtualizzazione tiene limitato il numero di righe renderizzate; il calcolo della topologia resta proporzionale ai commit caricati. L'accettazione nativa precedente ha verificato 5.011 commit reali caricati, circa 30 righe esposte nella zona visibile, navigazione, ricerca e menu: è evidenza funzionale, non un benchmark FPS.

## Confronto misurato con GitKraken: spazio del bundle

| Bundle locale arm64 | Dimensione logica file | Spazio allocato sul disco |
| --- | ---: | ---: |
| Ramo {footprint['ramo']['version']} | {ramo_mb:.2f} MB | {footprint['ramo']['diskAllocatedKiB']/1024:.2f} MiB |
| GitKraken {footprint['gitkraken']['version']} | {kraken_mb:.2f} MB | {footprint['gitkraken']['diskAllocatedKiB']/1024:.2f} MiB |

In questa installazione il bundle di Ramo contiene **{reduction:.1f}% di byte in meno** (rapporto {kraken_mb/ramo_mb:.1f}:1). MB = 1.000.000 byte; MiB = 1.048.576 byte. Dimensione logica: somma dei file senza seguire symlink; allocazione: `du -sk`. Entrambi gli eseguibili principali sono arm64. Ramo usa WebKit di sistema, GitKraken include Electron; funzionalità e dipendenze distribuite differiscono. Questo confronto **non dimostra minore RAM, minore CPU o maggiore velocità**. Non si confrontano un DMG compresso e un'app estratta.

## Controlli della preview e limiti

- {checks['frontendTests']} test frontend e {checks['backendTests']} backend (22 unit/trasporto, 86 Git, 6 processi) passati. I test dei comandi usano repository Git temporanei; nessuna modifica a repository personali.
- TypeScript/Vite production build e accettazione nativa dello stesso binario: menu, drag confronto/merge/cherry-pick, staging, editor conflitti, stash e worktree. Questi sono controlli di correttezza, non tempi di risposta.
- npm audit: **0 vulnerabilità note** al momento del controllo; non equivale a una revisione di sicurezza completa. Nessuna nuova inferenza AI live per questa consegna.
- DMG: checksum, montaggio in sola lettura, versione, shortcut Applicazioni, eseguibile, licenze e firma ad-hoc verificati; volume di prova smontato.
- Gatekeeper ha rifiutato la valutazione dell'app (`spctl`, exit {checks['gatekeeperAssessment']['exitCode']}). La firma è ad-hoc: **mancano Developer ID e notarizzazione Apple**. L'avvio dopo un download su un secondo Mac non è verificato. La guida include il riferimento ufficiale Apple per un'eventuale autorizzazione manuale della singola app.
- Niente installer Intel/Windows/Linux verificati. Non sono misurati rete remota, AI, avvio a freddo, RAM/CPU/energia, repository enormi o grafi complessi, grandi binari, checkout massivi o risoluzione umana dei conflitti.

## Cosa serve per una percentuale di velocità su GitKraken

Usare copie identiche di un fixture sintetico, stessa macchina e profilo energetico, stessa vista/quantità di dati ed estensioni disattivate equivalenti. Alternare casualmente l'ordine dei due client. Per ogni operazione definire l'inizio (input accettato dopo la conferma) e la fine osservabile (risultato corretto e interfaccia utilizzabile); separare avvio freddo e caldo. Raccogliere almeno 30 prove per client, compresi gli errori, con una registrazione temporale indipendente dei frame o strumentazione equivalente in entrambi. Cronometrare fuori dall'automazione di accessibilità: le sue attese e la lettura degli alberi UI hanno costi diversi nei due programmi. Riportare mediana, dispersione e condizioni; verificare HEAD/index/file prima e dopo. Fetch/push vanno confrontati separatamente su un remote controllato; l'AI richiede stesso modello e input e dipende dal provider.

Solo allora una riduzione percentuale del tempo, `(tempo GitKraken - tempo Ramo) / tempo GitKraken × 100`, descriverà quella specifica operazione. Oggi il rapporto di velocità è **non determinato**. Non si deduce dal framework Tauri/Electron o dal peso del bundle.

## Riproducibilità e dati

Nei sorgenti:

```sh
npm test
npm run test:git
npm run audit:dependencies
npm run bench:delivery
npm run bench:history
```

`bench:delivery` richiede le dipendenze Rust già disponibili in cache perché usa `--offline`; per il primo setup eseguire i normali comandi di build/test con accesso ai registry. Il benchmark crea e rimuove solo un fixture temporaneo, non usa rete o AI e non è un'app di produzione. Due esecuzioni preliminari di sviluppo sono conservate come `delivery-performance-initial.json` e `delivery-performance-repeat.json`: durante la prima era in corso anche il controllo interattivo di GitKraken e compaiono outlier oltre un secondo. La tabella usa l'ultima esecuzione del benchmark finale, che aggiunge anche controlli di unstage, messaggi commit distinti e la fase merge; non combina campioni di versioni diverse. Sono disponibili tutti i valori, non solo le mediane.

I file del pacchetto sono sotto `benchmark-data/`: `delivery-performance.json`, `delivery-history-render.json`, `delivery-environment.json`, `distribution-footprint.json`, `preview-validation.json` e le due esecuzioni preliminari. Nessun repository, screenshot di cliente, credenziale o percorso personale è distribuito.
'''
    report=intro+table+rest
    directory=ROOT/'docs/distribution'
    (directory/'REPORT-PRESTAZIONI.md').write_text(report)
    # Portable HTML: escaped prose, semantic tables, no network or scripts.
    lines=report.splitlines(); output=[]; in_code=False; table_open=False
    for line in lines:
      if line.startswith('```'):
        if in_code:output.append('</code></pre>')
        else:output.append('<pre><code>')
        in_code=not in_code;continue
      if in_code:output.append(html.escape(line)+'\n');continue
      if line.startswith('|'):
        if set(line.replace('|','').replace(':','').replace('-','').strip())==set():continue
        if not table_open:output.append('<div class="table-wrap"><table>');table_open=True
        cells=[html.escape(v.strip()) for v in line.strip('|').split('|')]
        output.append('<tr>'+''.join('<td>'+c+'</td>' for c in cells)+'</tr>');continue
      if table_open:output.append('</table></div>');table_open=False
      text=html.escape(line).replace('**','')
      if line.startswith('### '):output.append('<h3>'+text[4:]+'</h3>')
      elif line.startswith('## '):output.append('<h2>'+text[3:]+'</h2>')
      elif line.startswith('# '):output.append('<h1>'+text[2:]+'</h1>')
      elif line:output.append('<p>'+text+'</p>')
    charts=''.join(f'<div class="metric"><span>{html.escape(label)}</span><div><i style="width:{median/max(r[1] for r in rows)*100:.2f}%"></i></div><b>{median:.2f} ms</b></div>' for label,median,_,_ in rows)
    doc='''<!doctype html><html lang="it"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Ramo 0.1.0 — report prestazioni</title><style>
body{margin:0;background:#171b20;color:#dce4eb;font:16px/1.65 system-ui,sans-serif}main{max-width:1060px;margin:auto;padding:40px 24px}h1{font-size:36px;line-height:1.2}h2{margin-top:42px;color:#9ce1bf}p{max-width:95ch}table{border-collapse:collapse;width:100%;font-size:14px}td{padding:11px;border-bottom:1px solid #3c4954}tr:first-child{font-weight:650;background:#26333c}.table-wrap{overflow:auto}pre{background:#222d35;padding:20px;overflow:auto}.metric{display:grid;grid-template-columns:minmax(240px,1fr) 1fr 100px;gap:14px;align-items:center;margin:10px 0;font-size:13px}.metric div{height:9px;background:#29353d}.metric i{display:block;height:9px;background:#7bd8ac}.metric b{text-align:right}.note{padding:20px;border-left:3px solid #7bd8ac;background:#22342b}a{color:#9ce1bf}@media(max-width:650px){.metric{grid-template-columns:1fr 80px}.metric div{display:none}}@media print{body{background:white;color:#182128}main{padding:0}h2{color:#236b4b}td{border-color:#aaa}tr:first-child{background:#eee}.note{background:#eee}h2,table{break-inside:avoid}a{color:#236b4b}}
</style><main>'''+''.join(output[:3])+'''<section aria-label="Mediane backend"><h2>Mediane delle operazioni interne</h2><p class="note">Ramo soltanto · 9 campioni dopo riscaldamento · valori più bassi migliori. Questo grafico non confronta la velocità con GitKraken.</p>'''+charts+'</section>'+''.join(output[3:])+'</main></html>'
    (directory/'REPORT-PRESTAZIONI.html').write_text(doc)
    print('Generated report from 13 metrics × 9 samples, static render and local footprint evidence.')
if __name__=='__main__':main()
