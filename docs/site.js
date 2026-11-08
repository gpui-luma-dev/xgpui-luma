// Shared documentation chrome and interactions. Works on file:// and GitHub Pages.
(() => {
    const header = document.querySelector('[data-site-header]');
    if (header) header.innerHTML = `<a class="brand" href="index.html" aria-label="Luma SDK documentation"><i class="mark" aria-hidden="true"></i>Luma<span>Documentation</span></a>
<div class="header-right"><a href="https://github.com/gpui-luma-dev/xgpui-luma">GitHub ↗</a><button class="theme" id="theme" aria-pressed="false" aria-label="Switch to dark theme"><svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.4" aria-hidden="true"><circle cx="10" cy="10" r="7"/><path d="M10 3a7 7 0 0 1 0 14Z" fill="currentColor" stroke="none"/></svg><span id="theme-label">Dark mode</span></button></div>`;

    const footer = document.querySelector('[data-site-footer]');
    if (footer) {
        const context = document.createElement('span');
        context.textContent = `Luma SDK · ${document.title.split(' · ')[0]}`;
        const attribution = document.createElement('span');
        attribution.textContent = 'Documentation created by Codex';
        footer.replaceChildren(context, attribution);
    }

// Keep navigation at the same position across full-page loads and history visits.
const sidebar = document.querySelector('.sidebar');
if (sidebar) {
    const sidebarScrollKey = 'luma-docs-sidebar-scroll';
    function restoreSidebarScroll() {
        if (!sidebar.clientWidth) return;
        try {
            const saved = sessionStorage.getItem(sidebarScrollKey);
            if (saved !== null && Number.isFinite(Number(saved))) {
                sidebar.scrollTop = Math.max(0, Number(saved));
            }
        } catch {
            // Navigation still works when session storage is unavailable.
        }
    }
    function saveSidebarScroll() {
        if (!sidebar.clientWidth) return;
        try { sessionStorage.setItem(sidebarScrollKey, String(sidebar.scrollTop)); } catch {}
    }
    restoreSidebarScroll();
    window.addEventListener('pagehide', saveSidebarScroll);
    window.addEventListener('pageshow', restoreSidebarScroll);
}

const root = document.documentElement;
const theme = document.getElementById('theme');
const systemTheme = window.matchMedia('(prefers-color-scheme: dark)');
function isDarkTheme() {
    return root.dataset.theme === 'dark' || (!root.dataset.theme && systemTheme.matches);
}
function syncTheme() {
    const dark = isDarkTheme();
    theme.setAttribute('aria-pressed', String(dark));
    theme.setAttribute('aria-label', dark ? 'Switch to light theme' : 'Switch to dark theme');
    document.getElementById('theme-label').textContent = dark ? 'Light mode' : 'Dark mode';
    if (location.protocol === 'file:') return;
    document.querySelectorAll('[data-luma-example]').forEach(frame => {
        frame.contentWindow?.postMessage({ type: 'luma-theme', dark }, location.origin);
    });
}
window.addEventListener('message', event => {
    if (event.origin !== location.origin || event.data?.type !== 'luma-ready') return;
    const frame = [...document.querySelectorAll('[data-luma-example]')]
        .find(frame => frame.contentWindow === event.source);
    if (frame) event.source.postMessage({ type: 'luma-theme', dark: isDarkTheme() }, location.origin);
});
syncTheme();
systemTheme.addEventListener('change', syncTheme);
theme.addEventListener('click', () => {
    const next = isDarkTheme() ? 'light' : 'dark';
    root.dataset.theme = next;
    try { localStorage.setItem('luma-docs-theme', next); } catch {}
    syncTheme();
});
// Highlight only explicitly marked code blocks; copying still reads plain text.
document.querySelectorAll('[data-example-tabs]').forEach(group => {
    const tabs = [...group.querySelectorAll('[role="tab"]')];
    const panel = group.querySelector('[role="tabpanel"]');
    const frame = panel.querySelector('[data-luma-example]');
    const description = panel.querySelector('[data-example-description]');
    const link = panel.querySelector('[data-example-link]');
    let restoreTabFocus = false;
    function select(tab) {
        if (tab.getAttribute('aria-selected') === 'true') return;
        tabs.forEach(item => {
            item.setAttribute('aria-selected', String(item === tab));
            item.tabIndex = item === tab ? 0 : -1;
        });
        panel.setAttribute('aria-labelledby', tab.id);
        description.textContent = tab.dataset.description;
        link.href = tab.dataset.src;
        frame.title = `Live ${tab.textContent} chart sample`;
        restoreTabFocus = true;
        // Navigate the existing iframe: its previous WASM document is released.
        frame.src = tab.dataset.src;
    }
    window.addEventListener('message', event => {
        if (event.origin !== location.origin || event.source !== frame.contentWindow ||
            event.data?.type !== 'luma-ready' || !restoreTabFocus) return;
        restoreTabFocus = false;
        // GPUI focuses its canvas on startup. Keep keyboard navigation in the
        // tab strip when switching examples, unless the user moved elsewhere.
        if (document.activeElement === frame || tabs.includes(document.activeElement)) {
            tabs.find(tab => tab.getAttribute('aria-selected') === 'true').focus({ preventScroll: true });
        }
    });
    tabs.forEach((tab, index) => {
        tab.addEventListener('click', () => select(tab));
        tab.addEventListener('keydown', event => {
            let next;
            if (event.key === 'ArrowRight') next = (index + 1) % tabs.length;
            else if (event.key === 'ArrowLeft') next = (index + tabs.length - 1) % tabs.length;
            else if (event.key === 'Home') next = 0;
            else if (event.key === 'End') next = tabs.length - 1;
            else return;
            event.preventDefault();
            tabs[next].focus();
            select(tabs[next]);
        });
    });
});
if (window.Prism) window.Prism.highlightAll();
document.querySelectorAll('.copy').forEach(button => {
    button.addEventListener('click', async () => {
        const code = button.closest('.codebox').querySelector('code');
        try {
            await navigator.clipboard.writeText(code.textContent);
            button.textContent = 'Copied';
        } catch {
            const selection = window.getSelection();
            const range = document.createRange();
            range.selectNodeContents(code);
            selection.removeAllRanges();
            selection.addRange(range);
            button.textContent = 'Selected — press copy';
        }
        setTimeout(() => { button.textContent = 'Copy'; }, 2200);
    });
});
})();
