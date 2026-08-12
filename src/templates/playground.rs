pub fn render(version: &str, model: &str, base_url: &str) -> String {
    format!(r#"<!doctype html>
<html lang="ru" data-theme="dark">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Playground — OpenCode Proxy</title>
  <style>
    :root {{
      --font: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
      --radius: 8px;
      --bg: #0f0f13;
      --card: #1a1a22;
      --border: #2a2a35;
      --text: #e4e4e7;
      --text-dim: #92929e;
      --accent: #6366f1;
      --accent-hover: #5457e5;
      --green: #10b981;
      --red: #f43f5e;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: var(--font); background: var(--bg); color: var(--text);
      padding: 24px; max-width: 720px; margin: 0 auto; line-height: 1.5;
    }}
    h1 {{ font-size: 1.4rem; font-weight: 600; margin-bottom: 4px; }}
    .sub {{ color: var(--text-dim); font-size: .85rem; margin-bottom: 24px; }}
    .card {{ background: var(--card); border: 1px solid var(--border); border-radius: var(--radius); padding: 20px; margin-bottom: 16px; }}
    label {{ display: block; font-size: .8rem; font-weight: 500; margin-bottom: 4px; color: var(--text-dim); }}
    input, textarea {{
      width: 100%; padding: 10px 12px; background: var(--bg); border: 1px solid var(--border);
      border-radius: 6px; color: var(--text); font-size: .9rem; font-family: var(--font);
    }}
    input:focus, textarea:focus {{ outline: none; border-color: var(--accent); }}
    textarea {{ resize: vertical; min-height: 60px; font-family: 'JetBrains Mono', monospace; font-size: .85rem; }}
    .row {{ display: flex; gap: 12px; }}
    .row > * {{ flex: 1; }}
    .form-group {{ margin-bottom: 12px; }}
    .btn {{
      display: inline-flex; align-items: center; gap: 8px; padding: 10px 20px; border-radius: 6px; border: none;
      font-size: .9rem; font-weight: 500; cursor: pointer; background: var(--accent); color: #fff;
    }}
    .btn:hover {{ background: var(--accent-hover); }}
    .btn:disabled {{ opacity: .5; cursor: not-allowed; }}
    .btn-group {{ display: flex; gap: 8px; margin-top: 4px; }}
    .spinner {{ display: inline-block; width: 16px; height: 16px; border: 2px solid rgba(255,255,255,.3); border-top-color: #fff; border-radius: 50%; animation: spin .6s linear infinite; }}
    @keyframes spin {{ to {{ transform: rotate(360deg); }} }}
    #result {{ display: none; }}
    #result pre {{ background: #0a0a0f; border: 1px solid var(--border); border-radius: 6px; padding: 12px; font-size: .8rem; overflow-x: auto; white-space: pre-wrap; word-break: break-all; max-height: 400px; overflow-y: auto; }}
    .badge {{ display: inline-block; padding: 2px 8px; border-radius: 20px; font-size: .75rem; font-weight: 500; }}
    .badge.ok {{ background: rgba(16,185,129,.15); color: var(--green); }}
    .badge.err {{ background: rgba(244,63,94,.15); color: var(--red); }}
    .status-bar {{ display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }}
    .header-link {{ color: var(--text-dim); font-size: .8rem; text-decoration: none; margin-left: auto; }}
    .header-link:hover {{ color: var(--accent); }}
    #error {{ display: none; color: var(--red); font-size: .85rem; margin-top: 8px; padding: 8px 12px; background: rgba(244,63,94,.1); border-radius: 6px; border: 1px solid rgba(244,63,94,.2); }}
  </style>
</head>
<body>
  <div style="display:flex;align-items:center;gap:12px;margin-bottom:4px">
    <h1>Playground</h1>
    <a href="/dashboard" class="header-link">Dashboard</a>
  </div>
  <p class="sub">v{version} — проверь, отвечает ли модель, без сохранения данных</p>

  <div class="card">
    <div class="row">
      <div class="form-group">
        <label for="model">Модель</label>
        <input type="text" id="model" value="{model}" placeholder="gpt-5">
      </div>
      <div class="form-group">
        <label for="baseUrl">Base URL</label>
        <input type="text" id="baseUrl" value="{base_url}" placeholder="https://api.example.com/v1">
      </div>
    </div>
    <div class="form-group">
      <label for="apiKey">API Key</label>
      <input type="password" id="apiKey" placeholder="sk-..." autocomplete="off">
    </div>
    <div class="form-group">
      <label for="prompt">Промпт (проверочный запрос)</label>
      <textarea id="prompt" rows="2">Привет! Ответь коротко: 1+1=?</textarea>
    </div>
    <div class="btn-group">
      <button class="btn" id="testBtn" onclick="testModel()">Отправить</button>
      <label style="display:flex;align-items:center;gap:6px;font-size:.85rem;color:var(--text-dim);cursor:pointer;margin:0">
        <input type="checkbox" id="streamCheck" checked> Потоковый (stream)
      </label>
    </div>
    <div id="error"></div>
  </div>

  <div id="result" class="card">
    <div class="status-bar">
      <span class="badge" id="statusBadge">200</span>
      <span style="font-size:.8rem;color:var(--text-dim)" id="statusText"></span>
      <span style="font-size:.8rem;color:var(--text-dim)" id="timeText"></span>
    </div>
    <pre id="responseBody"></pre>
  </div>

  <script>
    async function testModel() {{
      const btn = document.getElementById('testBtn');
      const result = document.getElementById('result');
      const error = document.getElementById('error');
      const body = document.getElementById('responseBody');
      const badge = document.getElementById('statusBadge');
      const statusText = document.getElementById('statusText');
      const timeText = document.getElementById('timeText');

      error.style.display = 'none';
      result.style.display = 'none';
      btn.disabled = true;
      btn.innerHTML = '<span class=\"spinner\"></span> Отправка...';

      const model = document.getElementById('model').value.trim();
      const baseUrl = document.getElementById('baseUrl').value.trim().replace(/\\\\/+$/, '');
      const ak = document.getElementById('apiKey').value.trim();
      const prompt = document.getElementById('prompt').value.trim() || 'ping';
      const stream = document.getElementById('streamCheck').checked;

      if (!model || !baseUrl || !ak) {{
        error.textContent = 'Заполни модель, URL и ключ.';
        error.style.display = 'block';
        btn.disabled = false;
        btn.innerHTML = 'Отправить';
        return;
      }}

      const startedAt = Date.now();
      try {{
        const res = await fetch('/playground/test', {{
          method: 'POST',
          headers: {{ 'Content-Type': 'application/json' }},
          body: JSON.stringify({{ model, baseUrl, apiKey: ak, prompt, stream }}),
        }});
        const elapsed = Date.now() - startedAt;
        timeText.textContent = elapsed + 'ms';
        badge.textContent = res.status;
        badge.className = 'badge ' + (res.ok ? 'ok' : 'err');
        statusText.textContent = res.ok ? 'OK' : 'Ошибка';

        const text = await res.text();
        try {{ body.textContent = JSON.stringify(JSON.parse(text), null, 2); }} catch {{ body.textContent = text; }}
        result.style.display = 'block';
      }} catch (err) {{
        error.textContent = 'Ошибка соединения: ' + err.message;
        error.style.display = 'block';
      }} finally {{
        btn.disabled = false;
        btn.innerHTML = 'Отправить';
      }}
    }}
  </script>
</body>
</html>"#)
}
