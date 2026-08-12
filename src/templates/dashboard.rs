pub fn render(version: &str) -> String {
    format!(r#"<!doctype html>
<html lang="ru" data-theme="dark">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>OpenCode Proxy Dashboard</title>
  <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.7/dist/chart.umd.min.js" integrity="sha384-vsrfeLOOY6KuIYKDlmVH5UiBmgIdB1oEf7p01YgWHuqmOHfZr374+odEv96n9tNC" crossorigin="anonymous" referrerpolicy="no-referrer"></script>
  <style>
    :root {{
      --font: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
      --radius: 12px; --radius-sm: 8px; --radius-xs: 6px;
      --shadow: 0 1px 3px rgba(0,0,0,.06), 0 1px 2px rgba(0,0,0,.04);
      --transition: 200ms cubic-bezier(.4,0,.2,1);
    }}
    [data-theme="dark"] {{
      --bg: #0f1117; --bg-card: #1c1f2e; --bg-card-hover: #232640;
      --bg-input: #252840; --bg-badge: #252840;
      --border: #2a2d42; --border-hover: #3d4166;
      --text: #e8eaed; --text-sec: #9ca3af; --text-muted: #6b7280;
      --accent: #6366f1; --accent-soft: rgba(99,102,241,.12);
      --good: #10b981; --good-bg: rgba(16,185,129,.12);
      --bad: #f43f5e; --bad-bg: rgba(244,63,94,.12);
      --warn: #f59e0b; --warn-bg: rgba(245,158,11,.12);
      --chart-grid: rgba(255,255,255,.04); --chart-text: #9ca3af;
    }}
    [data-theme="light"] {{
      --bg: #f8fafc; --bg-card: #ffffff; --bg-card-hover: #f1f5f9;
      --bg-input: #f1f5f9; --bg-badge: #f1f5f9;
      --border: #e2e8f0; --border-hover: #cbd5e1;
      --text: #0f172a; --text-sec: #475569; --text-muted: #94a3b8;
      --accent: #6366f1; --accent-soft: rgba(99,102,241,.08);
      --good: #059669; --good-bg: rgba(5,150,105,.08);
      --bad: #e11d48; --bad-bg: rgba(225,29,72,.08);
      --warn: #d97706; --warn-bg: rgba(217,119,6,.08);
      --chart-grid: rgba(0,0,0,.05); --chart-text: #64748b;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{ font-family: var(--font); background: var(--bg); color: var(--text); line-height: 1.5; -webkit-font-smoothing: antialiased; }}
    .container {{ max-width: 1440px; margin: 0 auto; padding: 24px; }}
    header {{ display: flex; align-items: center; justify-content: space-between; padding: 16px 0; }}
    header h1 {{ font-size: 24px; font-weight: 700; }}
    .header-actions {{ display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }}
    .version {{ font-size: 12px; color: var(--text-muted); }}
    .version a {{ color: var(--text-muted); text-decoration: none; }}
    .version a:hover {{ color: var(--text); }}
    .btn {{
      display: inline-flex; align-items: center; gap: 4px; padding: 8px 14px;
      border-radius: var(--radius-sm); font-size: 13px; font-weight: 500;
      border: 1px solid var(--border); background: var(--bg-card);
      color: var(--text); cursor: pointer; transition: all var(--transition); text-decoration: none;
    }}
    .btn:hover {{ border-color: var(--border-hover); background: var(--bg-card-hover); }}
    .btn-primary {{ background: var(--accent); color: #fff; border-color: var(--accent); }}
    .btn-primary:hover {{ opacity: .9; border-color: var(--accent); }}
    .health-banner {{ padding: 12px 20px; font-size: 13px; display: flex; align-items: center; gap: 12px; border-radius: var(--radius-sm); margin-bottom: 16px; flex-direction: column; align-items: flex-start; }}
    .stat-row {{ display: grid; grid-template-columns: repeat(5, 1fr); gap: 12px; margin-bottom: 24px; }}
    .stat-card {{
      background: var(--bg-card); border: 1px solid var(--border);
      border-radius: var(--radius); padding: 16px; transition: all var(--transition);
    }}
    .stat-card:hover {{ border-color: var(--border-hover); }}
    .stat-card-top {{ display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 8px; }}
    .stat-icon {{ display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; border-radius: var(--radius-sm); background: var(--accent-soft); color: var(--accent); }}
    .stat-icon svg {{ width: 15px; height: 15px; }}
    .stat-icon.tokens {{ background: rgba(139,92,246,.12); color: #8b5cf6; }}
    .stat-icon.latency {{ background: var(--warn-bg); color: var(--warn); }}
    .stat-icon.errors {{ background: var(--bad-bg); color: var(--bad); }}
    .stat-icon.cost {{ background: var(--good-bg); color: var(--good); }}
    .stat-label {{ font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: .5px; color: var(--text-muted); }}
    .stat-value {{ font-size: 26px; font-weight: 700; letter-spacing: -.5px; line-height: 1; }}
    .stat-sub {{ font-size: 12px; color: var(--text-sec); margin-top: 6px; }}
    .charts-grid {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 16px; margin-bottom: 24px; }}
    .chart-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); padding: 20px; transition: all var(--transition); }}
    .chart-card:hover {{ border-color: var(--border-hover); }}
    .chart-card.wide {{ grid-column: span 2; }}
    .chart-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }}
    .chart-title-row {{ display: flex; align-items: center; gap: 10px; }}
    .chart-title {{ font-size: 14px; font-weight: 600; color: var(--text); }}
    .chart-subtitle {{ font-size: 11px; color: var(--text-muted); margin-top: 2px; }}
    .chart-icon {{ display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; border-radius: var(--radius-sm); background: var(--accent-soft); color: var(--accent); flex-shrink: 0; }}
    .chart-icon svg {{ width: 15px; height: 15px; }}
    .chart-tabs {{ display: flex; gap: 2px; background: var(--bg-input); border-radius: var(--radius-xs); padding: 2px; }}
    .chart-tab {{ padding: 4px 10px; font-size: 11px; font-weight: 500; border: none; background: transparent; color: var(--text-muted); cursor: pointer; border-radius: var(--radius-xs); transition: all var(--transition); font-family: var(--font); }}
    .chart-tab.active {{ background: var(--accent); color: #fff; }}
    .chart-tab:hover:not(.active) {{ color: var(--text); }}
    .chart-wrap {{ height: 200px; position: relative; }}
    .chart-wrap canvas {{ width: 100% !important; }}
    .donuts-grid {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 24px; }}
    .donut-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); padding: 20px; text-align: center; }}
    .donut-wrap {{ position: relative; width: 160px; height: 160px; margin: 0 auto 12px; }}
    .donut-center {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); text-align: center; pointer-events: none; }}
    .donut-center-value {{ font-size: 20px; font-weight: 700; line-height: 1; }}
    .donut-center-label {{ font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: .5px; }}
    .donut-legend {{ display: flex; flex-wrap: wrap; justify-content: center; gap: 6px 12px; margin-top: 12px; }}
    .legend-item {{ display: inline-flex; align-items: center; gap: 5px; font-size: 11px; color: var(--text-sec); }}
    .legend-dot {{ width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }}
    .section {{ display: flex; flex-direction: column; gap: 12px; margin-bottom: 24px; }}
    .section-header {{ display: flex; justify-content: space-between; align-items: center; }}
    .section-title {{ font-size: 16px; font-weight: 600; }}
    .table-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); overflow: hidden; }}
    table {{ width: 100%; border-collapse: collapse; }}
    th, td {{ padding: 10px 14px; text-align: left; font-size: 12px; border-bottom: 1px solid var(--border); }}
    th {{ color: var(--text-muted); font-weight: 600; text-transform: uppercase; font-size: 10px; letter-spacing: .3px; background: var(--bg-card); }}
    .right {{ text-align: right; }}
    .ok {{ color: var(--good); }} .fail {{ color: var(--bad); }} .warn {{ color: var(--warn); }} .muted {{ color: var(--text-muted); }}
    .empty-state {{ text-align: center; padding: 24px; color: var(--text-muted); font-size: 13px; }}
    .model-grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }}
    .model-card {{ background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius); padding: 16px; transition: all var(--transition); }}
    .model-card:hover {{ border-color: var(--border-hover); }}
    .model-card-header {{ display: flex; justify-content: space-between; align-items: flex-start; gap: 8px; margin-bottom: 12px; }}
    .model-name {{ font-size: 13px; font-weight: 600; word-break: break-all; line-height: 1.3; }}
    .badge {{ display: inline-flex; align-items: center; padding: 3px 8px; border-radius: 999px; font-size: 10px; font-weight: 600; text-transform: uppercase; letter-spacing: .3px; white-space: nowrap; flex-shrink: 0; }}
    .badge.available {{ background: var(--good-bg); color: var(--good); }}
    .badge.limited {{ background: var(--bad-bg); color: var(--bad); }}
    .badge.retry {{ background: var(--warn-bg); color: var(--warn); }}
    .badge.error {{ background: var(--bad-bg); color: var(--bad); }}
    .badge.untested {{ background: var(--bg-badge); color: var(--text-muted); }}
    .model-status-text {{ font-size: 18px; font-weight: 700; margin-bottom: 6px; }}
    .model-sub {{ font-size: 12px; color: var(--text-muted); min-height: 30px; }}
    .quota-bar {{ margin-top: 12px; }}
    .quota-row {{ display: flex; justify-content: space-between; font-size: 11px; color: var(--text-sec); margin-bottom: 4px; }}
    .quota-row strong {{ color: var(--text); font-weight: 600; }}
    .progress-track {{ height: 4px; background: var(--bg-input); border-radius: 999px; overflow: hidden; }}
    .progress-fill {{ height: 100%; border-radius: 999px; transition: width 600ms cubic-bezier(.4,0,.2,1); }}
    .progress-fill.good {{ background: var(--good); }} .progress-fill.bad {{ background: var(--bad); }}
    .progress-fill.unknown {{ background: repeating-linear-gradient(90deg, var(--border) 0 6px, transparent 6px 12px); }}
    .model-stats {{ display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 12px; }}
    .model-stat-label {{ font-size: 10px; color: var(--text-muted); text-transform: uppercase; letter-spacing: .3px; }}
    .model-stat-value {{ font-size: 13px; font-weight: 600; }}
    .status-pill {{ display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 999px; font-size: 12px; font-weight: 600; background: var(--good-bg); color: var(--good); border: 1px solid transparent; }}
    .status-pill.error {{ background: var(--bad-bg); color: var(--bad); }}
    .status-dot {{ width: 6px; height: 6px; border-radius: 50%; background: currentColor; animation: pulse 2s ease-in-out infinite; }}
    @keyframes pulse {{ 0%, 100% {{ opacity: 1; }} 50% {{ opacity: .4; }} }}
    .privacy-note {{ padding: 12px 16px; background: var(--bg-card); border: 1px solid var(--border); border-radius: var(--radius-sm); font-size: 12px; color: var(--text-muted); margin-bottom: 24px; }}
    @media (max-width: 1200px) {{ .model-grid {{ grid-template-columns: repeat(2, 1fr); }} .stat-row {{ grid-template-columns: repeat(3, 1fr); }} .donuts-grid {{ grid-template-columns: repeat(2, 1fr); }} }}
    @media (max-width: 768px) {{ .stat-row {{ grid-template-columns: 1fr 1fr; }} .charts-grid {{ grid-template-columns: 1fr; }} .chart-card.wide {{ grid-column: span 1; }} .donuts-grid {{ grid-template-columns: 1fr; }} .model-grid {{ grid-template-columns: 1fr; }} }}
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>OpenCode Proxy</h1>
      <div class="header-actions">
        <a class="btn" href="/playground">Playground</a>
        <div class="status-pill" id="status" role="status" aria-live="polite"><span class="status-dot"></span> Loading...</div>
        <a class="btn" href="/export/usage.csv?days=7" download title="Скачать CSV">CSV</a>
        <a class="btn" href="/export/usage.json?days=7" download title="Скачать JSON">JSON</a>
        <button class="btn" id="themeToggle" type="button" title="Сменить тему">🌓</button>
        <button class="btn btn-primary" id="refresh" type="button">⟳ Обновить</button>
      </div>
    </header>
    <div id="healthBanner" class="health-banner" style="display:none"></div>

    <div class="stat-row">
      <div class="stat-card"><div class="stat-card-top"><div class="stat-label">Запросы (5 мин)</div><div class="stat-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19V5"/><path d="M4 19h16"/><path d="M8 15l3-3 3 2 5-6"/></svg></div></div><div class="stat-value" id="requests">0</div><div class="stat-sub" id="rpm">0 rpm</div></div>
      <div class="stat-card"><div class="stat-card-top"><div class="stat-label">Токены сегодня</div><div class="stat-icon tokens"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2v20"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7H14a3.5 3.5 0 0 1 0 7H6"/></svg></div></div><div class="stat-value" id="tpm">0</div><div class="stat-sub" id="tokens">0 tokens</div></div>
      <div class="stat-card"><div class="stat-card-top"><div class="stat-label">Задержка</div><div class="stat-icon latency"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="13" r="8"/><path d="M12 9v4l3 2"/></svg></div></div><div class="stat-value" id="latency">0ms</div><div class="stat-sub" id="maxLatency">max 0ms</div></div>
      <div class="stat-card"><div class="stat-card-top"><div class="stat-label">Ошибки (5 мин)</div><div class="stat-icon errors"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.3 3.9L1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg></div></div><div class="stat-value" id="errors">0</div><div class="stat-sub" id="success">0 ok</div></div>
      <div class="stat-card"><div class="stat-card-top"><div class="stat-label">Стоимость</div><div class="stat-icon cost"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 1v22"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7H14a3.5 3.5 0 0 1 0 7H6"/></svg></div></div><div class="stat-value" id="cost">$0</div><div class="stat-sub" id="costToday">$0</div></div>
    </div>

    <div class="section"><div class="section-header"><div class="section-title">Наблюдаемые модели</div></div><div class="model-grid" id="modelCards"></div></div>
    <div class="section"><div class="section-header"><div class="section-title">Лимиты API</div></div><div class="table-card"><table><thead><tr><th>Модель</th><th>Статус</th><th>Остаток</th><th>Ошибка</th></tr></thead><tbody id="limits"><tr><td colspan="4" class="empty-state">Активных лимитов пока нет</td></tr></tbody></table></div></div>
    <div class="section"><div class="section-header"><div class="section-title">Провайдеры</div></div><div class="table-card"><table><thead><tr><th>Имя</th><th>URL</th><th>Состояние</th><th>Цепь</th><th class="right">Запросов</th><th class="right">Ошибок</th></tr></thead><tbody id="providers"><tr><td colspan="6" class="empty-state">Загрузка...</td></tr></tbody></table></div></div>
    <div class="section"><div class="section-header"><div class="section-title">Последние запросы</div></div><div class="table-card"><table><thead><tr><th>Время</th><th>Модель</th><th>Статус</th><th class="right">Задержка</th><th class="right">Токены</th></tr></thead><tbody id="recent"><tr><td colspan="5" class="empty-state">Запросов пока нет</td></tr></tbody></table></div></div>

    <div class="charts-grid">
      <div class="chart-card wide"><div class="chart-header"><div><div class="chart-title-row"><div class="chart-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M3 3v18h18"/><path d="M7 15l4-4 3 3 5-7"/></svg></div><div><div class="chart-title">Активность</div><div class="chart-subtitle">Запросы, OK/ошибки по минутам</div></div></div></div><div class="chart-tabs" id="activityTabs"><button class="chart-tab active" data-metric="requests">Запросы</button><button class="chart-tab" data-metric="tokens">Токены</button></div></div><div class="chart-wrap"><canvas id="activityChart"></canvas></div></div>
      <div class="chart-card"><div class="chart-header"><div><div class="chart-title-row"><div class="chart-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="13" r="8"/><path d="M12 9v4l3 2"/></svg></div><div><div class="chart-title">Задержка</div><div class="chart-subtitle">Средняя и пиковая</div></div></div></div></div><div class="chart-wrap"><canvas id="latencyChart"></canvas></div></div>
      <div class="chart-card"><div class="chart-header"><div><div class="chart-title-row"><div class="chart-icon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 19h16"/><path d="M7 16V8"/><path d="M12 16V4"/><path d="M17 16v-6"/></svg></div><div><div class="chart-title">Стоимость</div><div class="chart-subtitle">Cost по минутам</div></div></div></div></div><div class="chart-wrap"><canvas id="costChart"></canvas></div></div>
    </div>

    <div class="donuts-grid">
      <div class="donut-card"><div class="chart-title" style="margin-bottom:12px;justify-content:center">Распределение по моделям</div><div class="donut-wrap"><canvas id="modelDonut"></canvas><div class="donut-center"><div class="donut-center-value" id="modelDonutTotal">0</div><div class="donut-center-label">запр.</div></div></div><div class="donut-legend" id="modelLegend"></div></div>
      <div class="donut-card"><div class="chart-title" style="margin-bottom:12px">Prompt vs Completion</div><div class="donut-wrap"><canvas id="tokenDonut"></canvas><div class="donut-center"><div class="donut-center-value" id="tokenDonutTotal">0</div><div class="donut-center-label">токенов</div></div></div><div class="donut-legend" id="tokenLegend"></div></div>
      <div class="donut-card"><div class="chart-title" style="margin-bottom:12px;justify-content:center">Успешность</div><div class="donut-wrap"><canvas id="successDonut"></canvas><div class="donut-center"><div class="donut-center-value" id="successDonutTotal">0%</div><div class="donut-center-label">ok rate</div></div></div><div class="donut-legend" id="successLegend"></div></div>
    </div>

    <div class="section"><div class="section-header"><div class="section-title">Расход сегодня</div></div><div class="table-card"><table><thead><tr><th>Модель</th><th class="right">Запр.</th><th class="right ok">OK</th><th class="right fail">Ошибки</th><th class="right">Токены</th><th class="right">мс</th></tr></thead><tbody id="todayModels"><tr><td colspan="6" class="empty-state">Запросов сегодня ещё нет</td></tr></tbody></table></div></div>

    <div class="privacy-note" id="privacy">Prompts, responses, API keys, session IDs, and local paths are never stored.</div>
  </div>

  <script>
    const VERSION = '{version}';
    const $ = id => document.getElementById(id);
    const fmt = new Intl.NumberFormat('ru-RU', {{ maximumFractionDigits: 0 }});
    const fmtD = new Intl.NumberFormat('ru-RU', {{ maximumFractionDigits: 2 }});

    let chartInstances = {{}};
    const chartColors = ['#6366f1','#f43f5e','#10b981','#f59e0b','#3b82f6','#8b5cf6','#ec4899','#14b8a6','#f97316','#06b6d4'];
    function getChartColor(i) {{ if (i < chartColors.length) return chartColors[i]; return 'hsl(' + ((i * 137.508) % 360) + ',70%,60%)'; }}

    function formatShortNum(n) {{ if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M'; if (n >= 1000) return (n / 1000).toFixed(1) + 'k'; return fmt.format(n); }}
    function tokenText(item) {{ if (!item || !item.requests) return '0'; return formatShortNum(item.total_tokens || 0); }}
    function stateLabel(s) {{ return s === 'available' ? 'доступна' : s === 'limited' ? 'лимит' : s === 'error' ? 'ошибка' : 'нет данных'; }}

    function initCharts() {{
      if (typeof Chart === 'undefined') return;
      chartInstances.activity = new Chart($('activityChart'), {{ type: 'line', data: {{ labels: [], datasets: [] }}, options: {{ responsive: true, maintainAspectRatio: false, animation: {{ duration: 300 }}, plugins: {{ legend: {{ display: false }} }}, scales: {{ x: {{ grid: {{ color: 'rgba(255,255,255,.04)' }}, ticks: {{ font: {{ size: 10 }} }}, border: {{ display: false }} }}, y: {{ grid: {{ color: 'rgba(255,255,255,.04)' }}, ticks: {{ font: {{ size: 10 }} }}, border: {{ display: false }}, beginAtZero: true }} }} }} }});
      chartInstances.latency = new Chart($('latencyChart'), {{ type: 'line', data: {{ labels: [], datasets: [] }}, options: {{ responsive: true, maintainAspectRatio: false, animation: {{ duration: 300 }}, plugins: {{ legend: {{ display: false }} }}, scales: {{ x: {{ grid: {{ color: 'rgba(255,255,255,.04)' }}, ticks: {{ font: {{ size: 10 }} }}, border: {{ display: false }} }}, y: {{ grid: {{ color: 'rgba(255,255,255,.04)' }}, ticks: {{ font: {{ size: 10 }} }}, border: {{ display: false }}, beginAtZero: true }} }} }} }});
      chartInstances.cost = new Chart($('costChart'), {{ type: 'line', data: {{ labels: [], datasets: [] }}, options: {{ responsive: true, maintainAspectRatio: false, animation: {{ duration: 300 }}, plugins: {{ legend: {{ display: false }} }}, scales: {{ x: {{ grid: {{ color: 'rgba(255,255,255,.04)' }}, ticks: {{ font: {{ size: 10 }} }}, border: {{ display: false }} }}, y: {{ grid: {{ color: 'rgba(255,255,255,.04)' }}, ticks: {{ font: {{ size: 10 }} }}, border: {{ display: false }}, beginAtZero: true }} }} }} }});
      const donutOpts = {{ responsive: true, maintainAspectRatio: true, cutout: '65%', animation: {{ duration: 500 }}, plugins: {{ legend: {{ display: false }}, tooltip: {{ backgroundColor: 'rgba(0,0,0,.85)', padding: 8, cornerRadius: 6 }} }} }};
      chartInstances.modelDonut = new Chart($('modelDonut'), {{ type: 'doughnut', data: {{ labels: [], datasets: [{{ data: [], backgroundColor: chartColors, borderWidth: 0 }}] }}, options: donutOpts }});
      chartInstances.tokenDonut = new Chart($('tokenDonut'), {{ type: 'doughnut', data: {{ labels: ['Prompt', 'Completion'], datasets: [{{ data: [0, 0], backgroundColor: ['#6366f1', '#10b981'], borderWidth: 0 }}] }}, options: donutOpts }});
      chartInstances.successDonut = new Chart($('successDonut'), {{ type: 'doughnut', data: {{ labels: ['OK', 'Ошибки', '429'], datasets: [{{ data: [0, 0, 0], backgroundColor: ['#10b981', '#f43f5e', '#f59e0b'], borderWidth: 0 }}] }}, options: donutOpts }});
    }}

    let activityMode = 'requests';
    $('activityTabs').addEventListener('click', (e) => {{
      const tab = e.target.closest('.chart-tab');
      if (!tab) return;
      document.querySelectorAll('#activityTabs .chart-tab').forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      activityMode = tab.dataset.metric;
    }});

    function updateDonut(chartId, legendId, centerId, labels, data, colors) {{
      const total = data.reduce((a, b) => a + b, 0);
      $(centerId).textContent = chartId === 'successDonut' ? (total > 0 ? Math.round((data[0] || 0) / total * 100) + '%' : '0%') : formatShortNum(total);
      $(legendId).innerHTML = labels.map((label, i) => '<span class="legend-item"><span class="legend-dot" style="background:' + colors[i] + '"></span>' + label + ' ' + (total > 0 ? Math.round(data[i] / total * 100) : 0) + '%</span>').join('');
      const chart = chartInstances[chartId];
      if (!chart) return;
      chart.data.labels = labels; chart.data.datasets[0].data = data; chart.data.datasets[0].backgroundColor = colors; chart.update('none');
    }}

    async function refresh() {{
      try {{
        const res = await fetch('/metrics?window=300000&days=7', {{ cache: 'no-store' }});
        if (!res.ok) throw new Error('HTTP ' + res.status);
        const data = await res.json();
        const s = data.summary?.window || {{}};
        const all = data.summary?.all || {{}};
        const usage = data.usage || {{}};
        const today = (usage.by_day || []).find(r => r.day === usage.today) || {{}};
        const ts = data.timeseries || [];

        $('status').innerHTML = '<span class="status-dot"></span> ' + new Date(data.generated_at).toLocaleTimeString() + ' · uptime ' + (data.uptime_seconds || 0) + 's';
        $('requests').textContent = fmt.format(s.requests || 0);
        $('rpm').textContent = fmtD.format(s.requests_per_minute || 0) + ' rpm';
        $('tpm').textContent = tokenText(today);
        $('tokens').textContent = '5 мин: ' + formatShortNum(s.tokens_per_minute || 0) + ' ток/мин';
        $('latency').textContent = fmt.format(s.latency_ms_avg || 0) + 'ms';
        $('maxLatency').textContent = 'max ' + fmt.format(s.latency_ms_max || 0) + 'ms';
        $('errors').textContent = fmt.format(s.fail || 0);
        $('success').textContent = fmt.format(s.ok || 0) + ' ok';
        $('cost').textContent = '$' + fmtD.format(today.cost || 0);
        $('costToday').textContent = '$' + fmtD.format(all.cost || 0) + ' total';

        const health = $('healthBanner');
        if (s.fail > s.ok) {{ health.style.display = 'flex'; health.style.background = 'var(--bad-bg)'; health.style.color = 'var(--bad)'; health.innerHTML = '<span>⚠️ Критическая ошибка: ' + fmt.format(s.fail) + ' из ' + fmt.format(s.requests) + '</span>'; }}
        else if (s.requests > 0 && s.fail > s.requests * 0.2) {{ health.style.display = 'flex'; health.style.background = 'var(--warn-bg)'; health.style.color = 'var(--warn)'; health.innerHTML = '<span>⚠️ Ошибок: ' + Math.round(s.fail/s.requests*100) + '%</span>'; }}
        else {{ health.style.display = 'flex'; health.style.background = 'var(--good-bg)'; health.style.color = 'var(--good)'; health.innerHTML = '<span>✅ Система работает</span>'; }}

        const filtered = (data.model_status?.all || []).slice(0, 8);
        $('modelCards').innerHTML = filtered.length === 0 ? '<div class="empty-state">Нет моделей</div>' : filtered.map(m => {{
          const st = m.state || 'untested';
          const today2 = m.today || {{}};
          return '<div class="model-card"><div class="model-card-header"><div class="model-name">' + m.model + '</div><div class="badge ' + st + '">' + stateLabel(st) + '</div></div><div class="model-stats"><div><div class="model-stat-label">Сегодня</div><div class="model-stat-value">' + fmt.format(today2.requests || 0) + '</div></div><div><div class="model-stat-label">Токены</div><div class="model-stat-value">' + tokenText(today2) + '</div></div></div></div>';
        }}).join('');

        const limits = data.limits || [];
        $('limits').innerHTML = limits.length === 0 ? '<tr><td colspan="4" class="empty-state">Активных лимитов нет</td></tr>' : limits.map(l => '<tr><td>' + l.model + '</td><td class="' + (l.limited ? 'fail' : 'muted') + '">' + (l.limited ? 'лимит' : 'ранее') + '</td><td>' + (l.rate_limit_remaining || '—') + ' / ' + (l.rate_limit_limit || '—') + '</td><td>' + (l.error_type || '') + '</td></tr>').join('');

        $('todayModels').innerHTML = (today.requests || 0) === 0 ? '<tr><td colspan="6" class="empty-state">Запросов сегодня нет</td></tr>' : '<tr><td>today</td><td class="right">' + fmt.format(today.requests || 0) + '</td><td class="right ok">' + fmt.format(today.ok || 0) + '</td><td class="right fail">' + fmt.format(today.fail || 0) + '</td><td class="right">' + tokenText(today) + '</td><td class="right">' + fmt.format(today.latency_ms_avg || 0) + '</td></tr>';

        $('recent').innerHTML = (data.recent || []).length === 0 ? '<tr><td colspan="5" class="empty-state">Запросов нет</td></tr>' : data.recent.slice(0, 10).map(e => '<tr><td>' + new Date(e.ts).toLocaleTimeString() + '</td><td>' + e.model + '</td><td class="' + (e.ok ? 'ok' : 'fail') + '">' + e.status + '</td><td class="right">' + fmt.format(e.latency_ms) + 'ms</td><td class="right">' + tokenText(e) + '</td></tr>').join('');

        if (ts.length > 0 && chartInstances.activity) {{
          const labels = ts.slice(-60).map(b => new Date(b.ts).toLocaleTimeString());
          const act = chartInstances.activity;
          act.data.labels = labels;
          act.data.datasets = activityMode === 'requests' ? [
            {{ label: 'Всего', data: ts.slice(-60).map(b => b.requests), borderColor: '#6366f1', fill: true, backgroundColor: 'rgba(99,102,241,.15)' }},
            {{ label: 'OK', data: ts.slice(-60).map(b => b.ok), borderColor: '#10b981', fill: true, backgroundColor: 'rgba(16,185,129,.1)' }},
            {{ label: 'Ошибки', data: ts.slice(-60).map(b => b.fail), borderColor: '#f43f5e', fill: true, backgroundColor: 'rgba(244,63,94,.1)' }},
          ] : [
            {{ label: 'Всего', data: ts.slice(-60).map(b => b.total_tokens), borderColor: '#8b5cf6', fill: true, backgroundColor: 'rgba(139,92,246,.15)' }},
            {{ label: 'Prompt', data: ts.slice(-60).map(b => b.prompt_tokens), borderColor: '#6366f1', fill: true, backgroundColor: 'rgba(99,102,241,.1)' }},
            {{ label: 'Completion', data: ts.slice(-60).map(b => b.completion_tokens), borderColor: '#10b981', fill: true, backgroundColor: 'rgba(16,185,129,.1)' }},
          ];
          act.update('none');

          chartInstances.latency.data.labels = labels;
          chartInstances.latency.data.datasets = [
            {{ label: 'Avg', data: ts.slice(-60).map(b => b.latency_ms_avg), borderColor: '#f59e0b', fill: true, backgroundColor: 'rgba(245,158,11,.1)' }},
            {{ label: 'Max', data: ts.slice(-60).map(b => b.latency_ms_max), borderColor: '#f43f5e', fill: false, borderDash: [4, 4] }},
          ];
          chartInstances.latency.update('none');

          let cumulative = 0;
          chartInstances.cost.data.labels = labels;
          chartInstances.cost.data.datasets = [
            {{ label: '$', data: ts.slice(-60).map(b => (cumulative += b.cost || 0, Math.round(cumulative * 1e6) / 1e6)), borderColor: '#10b981', fill: true, backgroundColor: 'rgba(16,185,129,.12)' }},
          ];
          chartInstances.cost.update('none');
        }}

        const modelAgg = {{}};
        for (const b of ts) {{ for (const [model, agg] of Object.entries(b.by_model || {{}})) {{ if (!modelAgg[model]) modelAgg[model] = 0; modelAgg[model] += agg.requests || 0; }} }}
        const ml = Object.keys(modelAgg).sort((a, b) => modelAgg[b] - modelAgg[a]);
        updateDonut('modelDonut', 'modelLegend', 'modelDonutTotal', ml, ml.map(m => modelAgg[m]), ml.map((_, i) => getChartColor(i)));

        let tP = today.prompt_tokens || 0, tC = today.completion_tokens || 0;
        if (!today.requests) {{ for (const b of ts) {{ tP += b.prompt_tokens || 0; tC += b.completion_tokens || 0; }} }}
        updateDonut('tokenDonut', 'tokenLegend', 'tokenDonutTotal', ['Prompt', 'Completion'], [tP, tC], ['#6366f1', '#10b981']);

        let tOk = today.ok || 0, tFail = today.fail || 0, tRL = today.rate_limited || 0;
        if (!today.requests) {{ for (const b of ts) {{ tOk += b.ok || 0; tFail += b.fail || 0; tRL += b.rate_limited || 0; }} }}
        updateDonut('successDonut', 'successLegend', 'successDonutTotal', ['OK', 'Ошибки', '429'], [tOk, tFail, tRL], ['#10b981', '#f43f5e', '#f59e0b']);
      }} catch (err) {{
        $('healthBanner').style.display = 'flex'; $('healthBanner').style.background = 'var(--bad-bg)'; $('healthBanner').style.color = 'var(--bad)';
        $('healthBanner').innerHTML = '<span>⚠️ ' + err.message + '</span>';
      }}
    }}

    $('themeToggle').addEventListener('click', () => {{ const h = document.documentElement; const n = h.getAttribute('data-theme') === 'dark' ? 'light' : 'dark'; h.setAttribute('data-theme', n); localStorage.setItem('oc-dash-theme', n); }});
    const saved = localStorage.getItem('oc-dash-theme');
    if (saved === 'light' || saved === 'dark') document.documentElement.setAttribute('data-theme', saved);
    initCharts(); refresh(); setInterval(refresh, 5000);
  </script>
</body>
</html>"#)
}
