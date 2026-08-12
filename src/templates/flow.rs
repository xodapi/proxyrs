pub fn render(version: &str) -> String {
    let h = format!(
        r##"<!doctype html>
<html lang="ru" data-theme="dark">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>OpenCode — Request Flow</title>
  <style>
    :root {{
      --font: 'Inter',system-ui,sans-serif; --radius: 12px; --radius-sm: 8px;
    }}
    [data-theme="dark"] {{
      --bg: #0f1117; --bg-card: #1c1f2e; --bg-card-hover: #232640;
      --border: #2a2d42; --text: #e8eaed; --text-sec: #9ca3af; --text-muted: #6b7280;
      --accent: #6366f1; --good: #10b981; --bad: #f43f5e; --warn: #f59e0b;
      --pipe-bg: #13151e; --node-bg: #1c1f2e; --node-border: #2a2d42;
    }}
    [data-theme="light"] {{
      --bg: #f8fafc; --bg-card: #ffffff; --bg-card-hover: #f1f5f9;
      --border: #e2e8f0; --text: #0f172a; --text-sec: #475569; --text-muted: #94a3b8;
      --accent: #6366f1; --good: #059669; --bad: #e11d48; --warn: #d97706;
      --pipe-bg: #f1f5f9; --node-bg: #ffffff; --node-border: #e2e8f0;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{ font-family: var(--font); background: var(--bg); color: var(--text); height: 100vh; overflow: hidden; }}
    .app {{ display: flex; flex-direction: column; height: 100vh; }}
    header {{ display: flex; align-items: center; justify-content: space-between; padding: 10px 20px; background: var(--bg-card); border-bottom: 1px solid var(--border); }}
    header h1 {{ font-size: 15px; font-weight: 600; }}
    .header-actions {{ display: flex; align-items: center; gap: 8px; }}
    .btn {{
      display: inline-flex; align-items: center; gap: 4px; padding: 5px 10px; border: 1px solid var(--border);
      border-radius: 6px; background: var(--bg-card); color: var(--text-sec); font-size: 12px;
      cursor: pointer; text-decoration: none;
    }}
    .btn:hover {{ background: var(--bg-card-hover); border-color: var(--accent); color: var(--text); }}
    .pipe-wrap {{ flex: 1; position: relative; overflow: hidden; background: var(--pipe-bg); }}
    .pipe-svg {{ position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; z-index: 1; }}
    .pipe-node {{
      position: absolute; z-index: 2; padding: 10px 16px; border-radius: var(--radius);
      background: var(--node-bg); border: 1.5px solid var(--node-border);
      min-width: 120px; text-align: center;
    }}
    .pipe-node:hover {{ border-color: var(--accent); }}
    .pipe-node .icon {{ font-size: 22px; }} .pipe-node .title {{ font-size: 12px; font-weight: 600; }}
    .pipe-node .metric {{ font-size: 11px; color: var(--text-muted); }}
    .pipe-node.active {{ border-color: var(--good); }} .pipe-node.error {{ border-color: var(--bad); }} .pipe-node.warn {{ border-color: var(--warn); }}
    .health-banner {{ padding: 6px 20px; font-size: 12px; display: flex; align-items: center; gap: 12px; background: var(--good); color: #fff; }}
    .health-banner.warn {{ background: var(--warn); }} .health-banner.err {{ background: var(--bad); }}
    .status-bar {{ display: flex; align-items: center; gap: 16px; padding: 6px 20px; background: var(--bg-card); border-top: 1px solid var(--border); font-size: 11px; color: var(--text-sec); }}
    .status-dot {{ width: 7px; height: 7px; border-radius: 50%; display: inline-block; background: var(--good); }}
    .loading {{ display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-muted); }}
  </style>
</head>
<body>
  <div class="app">
    <header>
      <h1>Request Flow</h1>
      <div class="header-actions">
        <a class="btn" href="/dashboard">Dashboard</a>
        <button class="btn" id="themeToggle">T</button>
      </div>
    </header>
    <div class="health-banner" id="healthBanner">Loading...</div>
    <div class="pipe-wrap" id="pipeWrap">
      <svg class="pipe-svg" id="pipeSvg"></svg>
      <div id="pipeNodes"></div>
    </div>
    <div class="status-bar">
      <span id="status"><span class="status-dot"></span> Loading...</span>
      <span id="statsLine"></span>
    </div>
  </div>
  <script>
    const V = '{version}';
    const $ = id => document.getElementById(id);
    const fmt = new Intl.NumberFormat('en-US', {{ maximumFractionDigits: 0 }});

    function drawEdges(edges) {{
      window._lastEdges = edges;
      const svg = $('pipeSvg');
      const wrap = $('pipeWrap');
      const r = wrap.getBoundingClientRect();
      let defs = '<defs><marker id="a" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="8" markerHeight="8" orient="auto"><path d="M0 0L10 5L0 10z" fill="#6366f1"/></marker></defs>';
      for (const e of edges) {{
        const from = document.getElementById(e.f);
        const to = document.getElementById(e.t);
        if (!from || !to) continue;
        const fr = from.getBoundingClientRect();
        const tr = to.getBoundingClientRect();
        const x1 = fr.left - r.left + fr.width/2;
        const y1 = fr.top - r.top + fr.height;
        const x2 = tr.left - r.left + tr.width/2;
        const y2 = tr.top - r.top;
        const d = e.d ? ' stroke-dasharray="5 4"' : '';
        defs += '<path d="M' + x1 + ' ' + y1 + ' L' + x2 + ' ' + y2 + '" stroke="' + (e.c || '#2a2d42') + '" stroke-width="' + (e.w || 1.5) + '" fill="none" marker-end="url(#a)"' + d + '/>';
      }}
      svg.innerHTML = defs;
    }}

    function render(data) {{
      const s = data.summary?.window || {{}};
      const all = data.summary?.all || {{}};
      const primary = data.model_status?.primary || [];
      const nodes = [
        {{ id:'n1', i:'C', t:'Client', m:fmt.format(s.requests||0)+' req', x:300, y:20 }},
        {{ id:'n2', i:'P', t:'Proxy', m:fmt.format(s.ok||0)+' OK', x:300, y:120 }},
        {{ id:'n3', i:'R', t:'Router', m:primary.length+' models', x:300, y:220 }},
        {{ id:'n4', i:'U', t:'Upstream', m:fmt.format(s.latency_ms_avg||0)+'ms', x:300, y:320, st:s.fail>s.ok?'error':'active' }},
        {{ id:'n5', i:'M', t:'Metrics', m:fmt.format(all.requests||0)+' total', x:300, y:420 }},
        {{ id:'n6', i:'OK', t:'Response', m:fmt.format(s.ok||0)+' OK', x:300, y:520 }},
      ];
      const nc = $('pipeNodes');
      nc.innerHTML = nodes.map(n => '<div class="pipe-node '+(n.st||'active')+'" id="'+n.id+'" style="left:'+n.x+'px;top:'+n.y+'px"><div class="icon">'+n.i+'</div><div class="title">'+n.t+'</div><div class="metric">'+n.m+'</div></div>').join('');
      const edges = [];
      for (let i=0; i<6; i++) if (i<5) edges.push({{f:'n'+(i+1), t:'n'+(i+2), c:'#6366f1', w:2}});
      for (let i=0; i<primary.length; i++) {{
        nc.innerHTML += '<div class="pipe-node" id="m'+i+'" style="left:'+(40+i*160)+'px;top:660px;min-width:auto;padding:4px 10px;border-radius:999px;font-size:11px">'+primary[i].model.slice(0,16)+'</div>';
        edges.push({{f:'n3', t:'m'+i, c:'#2a2d42', w:1, d:true}});
      }}
      requestAnimationFrame(() => drawEdges(edges));
      const hb = $('healthBanner');
      if (s.fail > s.ok) {{ hb.className = 'health-banner err'; hb.textContent = 'ERROR: '+fmt.format(s.fail)+' failures'; }}
      else if (s.requests > 0 && s.fail > s.requests * 0.2) {{ hb.className = 'health-banner warn'; hb.textContent = 'WARN: '+Math.round(s.fail/s.requests*100)+'% fail'; }}
      else {{ hb.className = 'health-banner'; hb.textContent = 'OK. '+fmt.format(s.requests||0)+' req/5min'; }}
    }}

    async function refresh() {{
      try {{
        const res = await fetch('/metrics?window=300000&days=1', {{ cache:'no-store' }});
        if (!res.ok) throw new Error('HTTP '+res.status);
        const data = await res.json();
        render(data);
        $('status').innerHTML = '<span class="status-dot"></span> '+(data.generated_at?new Date(data.generated_at).toLocaleTimeString():'--');
        const s = data.summary?.window||{{}};
        $('statsLine').textContent = fmt.format(s.requests||0)+' req '+fmt.format(s.ok||0)+' OK '+fmt.format(s.fail||0)+' fail';
      }} catch(e) {{
        $('status').innerHTML = 'ERR: '+e.message;
      }}
    }}

    $('themeToggle').onclick = () => {{
      const h = document.documentElement;
      const n = h.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
      h.setAttribute('data-theme', n);
      localStorage.setItem('oc-dash-theme', n);
    }};
    const saved = localStorage.getItem('oc-dash-theme');
    if (saved === 'light' || saved === 'dark') document.documentElement.setAttribute('data-theme', saved);
    window.addEventListener('resize', () => {{ if (window._lastEdges) requestAnimationFrame(() => drawEdges(window._lastEdges)); }});
    refresh();
    setInterval(refresh, 3000);
  </script>
</body>
</html>"##
    );
    h
}
