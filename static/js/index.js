// 主题控制
const themeBtn = document.getElementById('theme_btn');
const themeMenu = document.getElementById('theme_menu');
const themeOpts = document.querySelectorAll('.theme-opt');

const setTheme = (mode) => {
    const isDark = mode === 'dark' || (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
    localStorage.setItem('user-theme', mode);
    themeOpts.forEach(opt => {
        const dot = opt.querySelector('.dot');
        if(opt.dataset.mode === mode) dot.classList.add('bg-current');
        else dot.classList.remove('bg-current');
    });
};

themeBtn.onclick = (e) => { e.stopPropagation(); themeMenu.classList.toggle('show'); };
window.onclick = () => themeMenu.classList.remove('show');
themeOpts.forEach(opt => opt.onclick = () => setTheme(opt.dataset.mode));
window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (localStorage.getItem('user-theme') === 'system') setTheme('system');
});
setTheme(localStorage.getItem('user-theme') || 'system');

// 滚动跳转与动画
const go = (id) => {
    const target = document.getElementById(id);
    if (target) {
        target.scrollIntoView({ behavior: 'smooth', block: 'start' });
        history.replaceState(null, '', '#' + id);
    }
};
document.getElementById('btn_projects')?.addEventListener('click', e => { e.preventDefault(); go('projects'); });
document.getElementById('btn_about')?.addEventListener('click', e => { e.preventDefault(); go('about'); });

(() => {
    const io = new IntersectionObserver((entries) => {
        entries.forEach(ent => {
            if (ent.isIntersecting) ent.target.classList.add('in');
            else ent.target.classList.remove('in');
        });
    }, { threshold: 0.1 });
    document.querySelectorAll('.reveal').forEach(el => io.observe(el));
})();

// 健康检查
(async () => {
    const statusBg = document.getElementById('avatarStatusBg');
    if (!statusBg) return;
    try {
        const res = await fetch('/api/v1/health');
        const data = await res.json();
        if (data.status === 'ok') statusBg.classList.add('status-ok');
        else statusBg.classList.add('status-error');
    } catch (e) { statusBg.classList.add('status-error'); }
})();
