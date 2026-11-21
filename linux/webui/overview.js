// Native Linux overview, loaded after every shared WebUI script.
// It renders beside the shared #v2Home (hidden on Linux) and only uses globals the shared WebUI
// exposes: AppState, esc, $, V2UI, ModsHelpers and VersionModule. The transport is webview2-shim.js.
(() => {
  'use strict';
  const UI = () => window.V2UI;
  const ICON = () => window.V2UI.ICON;
  const ACTIONS = {
    linuxInstallLoader: '安装加载器',
    linuxUpdateLoader: '更新加载器',
    linuxUninstallLoader: '卸载加载器',
  };
  const state = { status: null, modStatus: null, pending: '', error: '', launchRequested: false };
  let root = null;

  const profiles = () => AppState.gameProfiles?.profiles || [];
  const activeGame = () => profiles().find(p => p.active);
  const gameValid = game => game?.directoryExists === true && game.valid === true;
  const canLaunch = game => !state.pending && !state.launchRequested && gameValid(game) &&
    state.status && state.status.gameRunning !== true;
  const modStatusOf = game => game && state.modStatus?.gameDirectory === game.directory ? state.modStatus : null;

  function loaderActions(game, status) {
    const version = state.status?.availableLoaderVersion;
    const installed = (status?.installedVersion || '').replace(/^v/i, '');
    const compare = installed && version ? window.ModsHelpers.compareVersion(installed, version) : null;
    const managed = status?.loaderManaged === true;
    const stopped = !state.pending && gameValid(game) && state.status?.gameRunning === false && status?.gameExeFound === true;
    return {
      version, managed, newer: compare > 0, repair: compare === 0,
      linuxInstallLoader: stopped && !!version && !managed && !status.loaderPresent && !status.installed,
      linuxUpdateLoader: stopped && !!version && managed && !(compare > 0),
      linuxUninstallLoader: stopped && managed,
    };
  }

  function loaderText(status) {
    if (!status) return '尚未确认加载器状态';
    if (status.loaderPresent && !status.loaderManaged) return '检测到不属于本工具的加载器文件';
    if (status.loaderDisabled) return 'CesiumLoader 已停用';
    if (status.loaderMissing) return 'CesiumLoader 文件缺失';
    if (!status.installed) return 'CesiumLoader 未安装';
    return `CesiumLoader ${status.installedVersion ? 'v' + status.installedVersion : '（版本未知）'}`;
  }

  function hero(game, badge, subtitle, launch) {
    return `<div class="v2-hero">
      <div class="v2-hero-art game-icon"><img src="images/astralparty-game-icon.png" alt="吉星派对"></div>
      <div class="v2-hero-text">
        <div class="v2-hero-title">吉星派对${game ? ` <span class="v2-tag">${esc(game.editionLabel || '')}</span>` : ''} ${badge}</div>
        <div class="v2-hero-sub">${esc(subtitle)}</div>
      </div>
      ${launch}
    </div>`;
  }

  function page(game) {
    const I = ICON();
    const status = modStatusOf(game);
    const { gameRunning, latestLog, summary, warning } = state.status || {};
    const ops = loaderActions(game, status);
    const info = window.VersionModule?.info;
    const version = info?.version ? `基于工具箱 v${info.version}${info.commit ? ` · ${info.commit}` : ''}` : '工具箱版本未读取';
    const heading = `<span class="v2-tag">Linux</span><span class="v2-muted">${esc(version)}</span><span class="v2-top-spacer"></span>
      <button type="button" class="v2-btn line" data-v2-linux="refresh" ${state.pending ? 'disabled' : ''}>${I.refresh}刷新状态</button>`;
    const choices = profiles();
    const picker = choices.length ? `<div class="v2-row"><label for="v2LinuxGame" class="k">当前游戏目录</label>
      <select class="v2-select v2-linux-game" id="v2LinuxGame" title="${esc(game?.directory)}" ${state.pending ? 'disabled' : ''}>
        ${game ? '' : '<option value="" selected disabled>请选择</option>'}
      </select></div>` : '';
    const badge = !game ? '' : !gameValid(game) ? '<span class="v2-badge warn">目录不可用</span>'
      : gameRunning === true ? '<span class="v2-badge pink">运行中</span>'
        : gameRunning === false ? '<span class="v2-badge off">未运行</span>' : '<span class="v2-badge warn">运行状态未确认</span>';
    const subtitle = game ? loaderText(status) : choices.length ? '选择要管理的安装目录'
      : AppState.gameProfiles ? '未找到游戏。请在 Steam 中安装吉星派对，然后刷新。' : '尚未读取 Steam 游戏目录';
    const launch = game ? `<button type="button" class="v2-btn primary big" data-v2-linux="launch" ${canLaunch(game) ? '' : 'disabled'}>${I.play}从 Steam 启动</button>` : '';
    const buttons = (ops.managed ? ['linuxUpdateLoader', 'linuxUninstallLoader'] : ['linuxInstallLoader'])
      .map(type => `<button type="button" class="v2-btn line" data-v2-linux="${type}" ${ops[type] ? '' : 'disabled'}>${ops.repair && type === 'linuxUpdateLoader' ? '修复加载器…' : `${ACTIONS[type]}…`}</button>`).join('');
    const mods = status && Array.isArray(status.mods) ? `${status.mods.filter(m => m.enabled === true).length} / ${status.mods.length} 配置为启用` : '未读取';
    const modified = latestLog?.modifiedUtc ? new Date(latestLog.modifiedUtc).toLocaleString('zh-CN') : '';
    const logLabel = latestLog ? [latestLog.name, modified].filter(Boolean).join(' · ') : state.status ? '未找到运行日志' : '尚未读取';
    const activity = state.pending || (state.launchRequested ? '启动请求已交给 Steam。可刷新运行状态和日志。' : '');
    return `<div class="v2-page"><div class="v2-head"><h1>概览</h1>${heading}</div>
      ${activity ? `<div class="v2-notice" role="status">${I.info}<span>${esc(activity)}</span></div>` : ''}
      ${state.error ? `<div class="v2-notice danger" role="alert">${I.info}<span>${esc(state.error)}</span></div>` : ''}
      <div class="v2-card">
        ${hero(game, badge, subtitle, launch)}
        ${picker}
        ${game ? '<div class="v2-row v2-muted">目录选择仅用于管理加载器和查看日志；启动时使用 Steam 当前配置的版本。</div>' : ''}
      </div>
      ${warning ? `<div class="v2-notice">${I.info}<span>${esc(warning)}</span></div>` : ''}
      ${game ? `<div class="v2-card">
        <div class="v2-row"><span class="k">磁盘模组状态</span><span class="v">${mods}</span></div>
        <div class="v2-row"><span class="k">支持的加载器</span><span class="v">${ops.version ? 'v' + esc(ops.version) : '未知'}${ops.newer ? '（已安装较高版本，不支持降级）' : ''}</span></div>
        <div class="v2-row v2-linux-actions">${buttons}${gameRunning === false ? '' : '<span class="v2-muted">确认游戏已关闭后刷新，才能更改加载器。</span>'}</div>
      </div>
      <div class="v2-card">
        <div class="v2-row"><span class="k">最近运行日志</span><span class="v" title="${esc(latestLog?.path)}">${esc(logLabel)}</span></div>
        ${summary ? `<div class="v2-row"><span class="k">日志统计</span><span class="v">加载 ${esc(summary.loaded ?? '未知')} · 失败 ${esc(summary.failed ?? '未知')}</span></div>` : ''}
        ${latestLog?.tail ? '<pre id="v2LinuxLog" class="v2-linux-log" tabindex="0" aria-label="最近运行日志末尾"></pre>' : ''}
        <div class="v2-row v2-muted">这里显示磁盘上的最新日志，不一定来自本次启动。游戏启动后可刷新查看。</div>
      </div>` : ''}
    </div>`;
  }

  function render() {
    if (!root) return;
    root.innerHTML = page(activeGame());
    const select = $('v2LinuxGame');
    if (select) {
      for (const profile of profiles()) {
        const option = new Option([profile.editionLabel, profile.directory].filter(Boolean).join(' · '),
          profile.id, profile.active, profile.active);
        option.disabled = !gameValid(profile);
        select.add(option);
      }
    }
    const log = $('v2LinuxLog');
    if (log) {
      log.textContent = state.status.latestLog.tail;
      log.scrollTop = log.scrollHeight;
    }
  }

  async function request(message, progress) {
    if (state.pending) return;
    state.pending = progress;
    state.error = '';
    // A read or Steam launch can invalidate an earlier "not running" observation.
    if (state.status) state.status = { ...state.status, gameRunning: null };
    render();
    try {
      await window.toysLinux.request(message);
      if (message.type === 'launchGame' && !state.error) state.launchRequested = true;
    } catch (error) {
      state.error = error?.message || String(error);
    } finally {
      state.pending = '';
      render();
    }
  }

  function confirmLoader(type) {
    const game = activeGame();
    const ops = loaderActions(game, modStatusOf(game));
    if (!ops[type]) return;
    const I = ICON();
    const removing = type === 'linuxUninstallLoader';
    const label = type === 'linuxUpdateLoader' && ops.repair ? '修复加载器' : ACTIONS[type];
    UI().modalActions.linuxLoaderYes = () => {
      UI().closeModal();
      const active = activeGame();
      if (active?.id === game.id && loaderActions(active, modStatusOf(active))[type]) {
        request({ type, id: game.id }, `正在${label}…`);
      }
    };
    const modal = UI().openModal(`
      <div class="v2-confirm"><div class="v2-confirm-icon">${removing ? I.trash : I.download}</div>
        <div><b class="v2-confirm-title">确认${label}？</b>
          <ul class="v2-confirm-lines"><li>仅操作当前游戏目录：<span class="v2-linux-target">${esc(game.directory)}</span></li>
            <li>${removing ? '只移除本工具安装且未被修改的文件；保留第三方模组、个人设置和日志。' : `安装受支持的 CesiumLoader v${esc(ops.version)}。`}</li>
            <li>${removing ? '如果 Proton 的 DLL 覆盖仍是本工具设置的值，将一并还原。' : '尝试为本游戏配置 Proton 的 DLL 覆盖，保留已有的用户设置。'}</li>
            ${!removing && modStatusOf(game)?.loaderDisabled ? '<li>加载器当前已停用，此操作不会重新启用。</li>' : ''}</ul></div></div>
      <div class="v2-modal-foot"><span class="v2-top-spacer"></span>
        <button type="button" class="v2-btn ghost" data-v2modal="close">取消</button>
        <button type="button" class="v2-btn ${removing ? 'danger-fill' : 'primary'}" data-v2modal="linuxLoaderYes">${label}</button></div>`, 'narrow');
    if (removing) UI().armDanger(modal);
  }

  function mount() {
    const shared = $('v2Home');
    if (!shared) return;
    root = document.createElement('div');
    root.id = 'linuxHome';
    shared.after(root);
    root.addEventListener('click', event => {
      const button = event.target.closest('[data-v2-linux]');
      if (!button || button.disabled) return;
      const action = button.dataset.v2Linux;
      if (action === 'refresh') request({ type: 'ready' }, '正在读取游戏与加载器状态…');
      else if (action === 'launch') {
        if (canLaunch(activeGame())) request({ type: 'launchGame', mode: 'steam' }, '正在请求 Steam 启动…');
      } else if (Object.hasOwn(ACTIONS, action)) {
        confirmLoader(action);
      }
    });
    root.addEventListener('change', event => {
      if (event.target.id !== 'v2LinuxGame') return;
      const profile = profiles().find(p => p.id === event.target.value);
      if (gameValid(profile) && !profile.active) request({ type: 'gameProfileSetActive', id: profile.id }, '正在切换游戏目录…');
    });
    request({ type: 'ready' }, '正在读取游戏与加载器状态…');
  }

  window.toysLinux.onMessage(msg => {
    switch (msg?.type) {
      case 'modStatus': state.modStatus = msg.payload?.status ?? msg.payload; break;
      case 'linuxStatus': state.status = msg.payload; state.launchRequested = false; break;
      case 'error': state.error = msg.message || '请求失败'; break;
      case 'gameProfiles': case 'appVersion': break;
      default: return;
    }
    render();
  });
  document.addEventListener('DOMContentLoaded', mount);
})();
