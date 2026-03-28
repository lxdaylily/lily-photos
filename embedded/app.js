const themeBtn = document.getElementById('theme_btn');
const themeMenu = document.getElementById('theme_menu');
const themeOpts = document.querySelectorAll('.theme-opt');
const form = document.getElementById('upload_form');
const fileInput = document.getElementById('photo');
const fileLabel = document.getElementById('file_label');
const formMsg = document.getElementById('form_msg');
const submitBtn = document.getElementById('submit_btn');
const galleryGrid = document.getElementById('gallery_grid');
const emptyState = document.getElementById('empty_state');
const healthDot = document.getElementById('health_dot');
const photoCountChip = document.getElementById('photo_count_chip');
const photoSizeChip = document.getElementById('photo_size_chip');
const authChip = document.getElementById('auth_chip');
const authTitle = document.getElementById('auth_title');
const authDesc = document.getElementById('auth_desc');
const adminPassword = document.getElementById('admin_password');
const adminPasswordConfirm = document.getElementById('admin_password_confirm');
const setupBtn = document.getElementById('setup_btn');
const loginBtn = document.getElementById('login_btn');
const logoutBtn = document.getElementById('logout_btn');
const uploadHint = document.getElementById('upload_hint');
const tagTemplate = document.getElementById('tag_template');
const photoModal = document.getElementById('photo_modal');
const modalCloseBtn = document.getElementById('modal_close_btn');
const modalImage = document.getElementById('modal_image');
const modalTime = document.getElementById('modal_time');
const modalTitle = document.getElementById('modal_title');
const modalDescription = document.getElementById('modal_description');

let isAdmin = false;
let currentPhotos = [];
let setupRequired = false;

const setTheme = (mode) => {
    const isDark =
        mode === 'dark' ||
        (mode === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);

    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
    localStorage.setItem('user-theme', mode);
    themeOpts.forEach((opt) => {
        opt.classList.toggle('active', opt.dataset.mode === mode);
    });
};

const colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');

themeBtn?.addEventListener('click', (event) => {
    event.stopPropagation();
    themeMenu?.classList.toggle('show');
});

window.addEventListener('click', () => themeMenu?.classList.remove('show'));
themeOpts.forEach((opt) => opt.addEventListener('click', () => setTheme(opt.dataset.mode)));

if (typeof colorSchemeQuery.addEventListener === 'function') {
    colorSchemeQuery.addEventListener('change', () => {
        if (localStorage.getItem('user-theme') === 'system') {
            setTheme('system');
        }
    });
} else if (typeof colorSchemeQuery.addListener === 'function') {
    colorSchemeQuery.addListener(() => {
        if (localStorage.getItem('user-theme') === 'system') {
            setTheme('system');
        }
    });
}

setTheme(localStorage.getItem('user-theme') || 'system');

fileInput?.addEventListener('change', () => {
    const files = Array.from(fileInput.files || []);
    if (files.length === 0) {
        fileLabel.textContent = '选择照片';
        return;
    }

    fileLabel.textContent = files.length === 1 ? files[0].name : `已选择 ${files.length} 张照片`;
});

const setMessage = (message, type = '') => {
    formMsg.textContent = message;
    formMsg.className = `form-msg ${type}`.trim();
};

const setAdminState = (authenticated, requiresSetup = false) => {
    isAdmin = authenticated;
    setupRequired = requiresSetup;
    authChip.textContent = authenticated ? '管理模式' : requiresSetup ? '等待初始化' : '访客模式';
    authTitle.textContent = authenticated ? '管理已登录' : requiresSetup ? '首次启动，先设置口令' : '管理未登录';
    authDesc.textContent = authenticated
        ? '现在可以上传新照片，也可以删除已有照片。'
        : requiresSetup
            ? '当前还没有管理员口令。先设置一次，之后用这个口令登录。'
            : '当前只能浏览照片，上传和删除需要管理口令。';
    uploadHint.textContent = authenticated
        ? '支持批量选择，照片会直接写入 SQLite 数据库。'
        : requiresSetup
            ? '先完成管理员口令初始化，之后才可以上传照片。'
            : '登录管理口令后才可以上传照片。';

    adminPassword.classList.toggle('hidden', authenticated);
    adminPasswordConfirm.classList.toggle('hidden', authenticated || !requiresSetup);
    setupBtn.classList.toggle('hidden', authenticated || !requiresSetup);
    loginBtn.classList.toggle('hidden', authenticated);
    loginBtn.textContent = requiresSetup ? '待初始化' : '登录';
    logoutBtn.classList.toggle('hidden', !authenticated);
    loginBtn.disabled = requiresSetup;
    submitBtn.disabled = !authenticated;
    form.querySelectorAll('input, textarea').forEach((node) => {
        if (node === adminPassword || node === adminPasswordConfirm) return;
        node.disabled = !authenticated;
    });
    form.classList.toggle('locked', !authenticated);
};

const formatBytes = (bytes) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const escapeHtml = (value) =>
    String(value)
        .replaceAll('&', '&amp;')
        .replaceAll('<', '&lt;')
        .replaceAll('>', '&gt;')
        .replaceAll('"', '&quot;')
        .replaceAll("'", '&#39;');

const formatDateParts = (createdAt) => {
    const raw = String(createdAt || '');
    const normalized = raw.includes('T') ? raw : raw.replace(' ', 'T');
    const utcNormalized =
        /(?:Z|[+-]\d{2}:\d{2})$/.test(normalized) || !normalized
            ? normalized
            : `${normalized}Z`;
    const date = new Date(utcNormalized);

    if (Number.isNaN(date.getTime())) {
        const [rawDate = '', rawTime = ''] = raw.split(' ');
        return {
            dateKey: rawDate || 'unknown',
            dayTitle: rawDate || '未知日期',
            daySub: rawTime ? `上传于 ${rawTime}` : '上传时间未知',
            timeLabel: rawTime || '未知时间',
            fullLabel: raw || '未知时间',
        };
    }

    return {
        dateKey: normalized.slice(0, 10),
        dayTitle: new Intl.DateTimeFormat('zh-CN', {
            month: 'long',
            day: 'numeric',
        }).format(date),
        daySub: new Intl.DateTimeFormat('zh-CN', {
            year: 'numeric',
            weekday: 'short',
        }).format(date),
        timeLabel: new Intl.DateTimeFormat('zh-CN', {
            hour: '2-digit',
            minute: '2-digit',
            hour12: false,
        }).format(date),
        fullLabel: new Intl.DateTimeFormat('zh-CN', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            weekday: 'long',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            hour12: false,
        }).format(date),
    };
};

const renderInlineMarkdown = (value) =>
    escapeHtml(value)
        .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
        .replace(/\*(.+?)\*/g, '<em>$1</em>')
        .replace(/`(.+?)`/g, '<code>$1</code>');

const renderMarkdown = (markdown) => {
    const source = String(markdown || '').replace(/\r\n/g, '\n').trim();
    if (!source) return '';

    return source
        .split(/\n{2,}/)
        .map((block) => {
            const line = block.trim();
            if (line.startsWith('#### ')) return `<h4>${renderInlineMarkdown(line.slice(5))}</h4>`;
            if (line.startsWith('### ')) return `<h3>${renderInlineMarkdown(line.slice(4))}</h3>`;
            if (line.startsWith('## ')) return `<h2>${renderInlineMarkdown(line.slice(3))}</h2>`;
            if (line.startsWith('# ')) return `<h1>${renderInlineMarkdown(line.slice(2))}</h1>`;
            return `<p>${renderInlineMarkdown(line).replace(/\n/g, '<br>')}</p>`;
        })
        .join('');
};

const openPhotoModal = (photo) => {
    const dateParts = formatDateParts(photo.created_at);
    modalImage.src = photo.content_url;
    modalImage.alt = photo.title || '照片';
    modalTime.textContent = dateParts.fullLabel;
    modalTitle.innerHTML = renderMarkdown(`## ${photo.title || '未命名照片'}`);
    modalDescription.innerHTML = renderMarkdown(
        photo.description ? `**${photo.description}**` : '**暂无描述**',
    );
    photoModal.classList.remove('hidden');
    photoModal.setAttribute('aria-hidden', 'false');
    document.body.style.overflow = 'hidden';
};

const closePhotoModal = () => {
    photoModal.classList.add('hidden');
    photoModal.setAttribute('aria-hidden', 'true');
    modalImage.src = '';
    document.body.style.overflow = '';
};

const renderPhotos = (photos) => {
    currentPhotos = photos;
    galleryGrid.innerHTML = '';
    emptyState.classList.toggle('hidden', photos.length > 0);
    const totalBytes = photos.reduce((sum, photo) => sum + photo.byte_size, 0);
    photoCountChip.textContent = `${photos.length} 张照片`;
    photoSizeChip.textContent = formatBytes(totalBytes);

    const grouped = new Map();
    photos.forEach((photo) => {
        const dateParts = formatDateParts(photo.created_at);
        const entry = grouped.get(dateParts.dateKey) || { ...dateParts, items: [] };
        entry.items.push(photo);
        grouped.set(dateParts.dateKey, entry);
    });

    [...grouped.values()].forEach((group) => {
        const section = document.createElement('section');
        section.className = 'timeline-day';
        section.innerHTML = `
            <div class="timeline-side">
                <div class="timeline-rail"></div>
                <div class="timeline-date">
                    <strong>${escapeHtml(group.dayTitle)}</strong>
                    <span>${escapeHtml(group.daySub)}</span>
                </div>
            </div>
            <div class="timeline-track"></div>
        `;

        const track = section.querySelector('.timeline-track');
        group.items.forEach((photo) => {
            const timeParts = formatDateParts(photo.created_at);
            const card = document.createElement('article');
            card.className = 'gallery-card';
            card.dataset.photoId = String(photo.id);
            const tagHtml = (photo.tags || [])
                .map((tag) => {
                    if (!tagTemplate) {
                        return `<span class="tag-chip">${escapeHtml(tag)}</span>`;
                    }
                    const node = tagTemplate.content.firstElementChild.cloneNode(true);
                    node.textContent = tag;
                    return node.outerHTML;
                })
                .join('');

            card.innerHTML = `
                <img src="${photo.content_url}" alt="${escapeHtml(photo.title)}">
                <div class="gallery-body">
                    <div class="card-head">
                        <div class="card-title-wrap">
                            <h3 class="card-title">${escapeHtml(photo.title)}</h3>
                            <div class="card-time">上传于 ${escapeHtml(timeParts.timeLabel)}</div>
                            ${tagHtml ? `<div class="tag-row">${tagHtml}</div>` : ''}
                        </div>
                    </div>
                    <div class="card-actions editor-actions ${isAdmin ? '' : 'hidden'}">
                        <button class="ghost-btn" type="button" data-edit-id="${photo.id}">编辑</button>
                        <button class="ghost-btn" type="button" data-delete-id="${photo.id}">删除</button>
                    </div>
                    <form class="editor-form" data-editor-id="${photo.id}">
                        <input name="title" type="text" maxlength="80" value="${escapeHtml(photo.title)}" placeholder="标题">
                        <textarea name="description" rows="3" maxlength="280" placeholder="描述">${escapeHtml(photo.description || '')}</textarea>
                        <input name="tags" type="text" maxlength="240" value="${escapeHtml((photo.tags || []).join(', '))}" placeholder="标签，用逗号分隔">
                        <div class="editor-actions">
                            <button class="ghost-btn solid" type="submit">保存</button>
                            <button class="ghost-btn" type="button" data-cancel-edit="${photo.id}">取消</button>
                        </div>
                    </form>
                </div>
            `;
            track.appendChild(card);
        });

        galleryGrid.appendChild(section);
    });
};

const loadPhotos = async () => {
    const response = await fetch('/api/v1/photos');
    if (!response.ok) {
        throw new Error('加载照片列表失败');
    }

    const payload = await response.json();
    renderPhotos(payload.items || []);
};

form?.addEventListener('submit', async (event) => {
    event.preventDefault();

    if (!isAdmin) {
        setMessage('请先登录管理口令。', 'error');
        return;
    }

    const photo = fileInput.files?.[0];
    if (!photo) {
        setMessage('请选择至少一张图片。', 'error');
        return;
    }

    const formData = new FormData(form);
    submitBtn.disabled = true;
    setMessage('正在写入数据库...');

    try {
        const response = await fetch('/api/v1/photos', {
            method: 'POST',
            body: formData,
        });
        const payload = await response.json();

        if (!response.ok) {
            throw new Error(payload.error || '上传失败');
        }

        form.reset();
        fileLabel.textContent = '选择照片';
        const count = payload.items?.length || 0;
        setMessage(`已写入数据库：${count} 张照片`, 'success');
        await loadPhotos();
    } catch (error) {
        setMessage(error.message || '上传失败', 'error');
    } finally {
        submitBtn.disabled = false;
    }
});

loginBtn?.addEventListener('click', async () => {
    const password = adminPassword.value.trim();
    if (!password) {
        setMessage('请输入管理口令。', 'error');
        return;
    }

    loginBtn.disabled = true;
    setMessage('正在验证管理口令...');

    try {
        const response = await fetch('/api/v1/auth/login', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ password }),
        });
        const payload = await response.json();

        if (!response.ok) {
            throw new Error(payload.error || '登录失败');
        }

        adminPassword.value = '';
        setAdminState(Boolean(payload.authenticated));
        await loadPhotos();
        setMessage('管理登录成功。', 'success');
    } catch (error) {
        setMessage(error.message || '登录失败', 'error');
    } finally {
        loginBtn.disabled = false;
    }
});

setupBtn?.addEventListener('click', async () => {
    const password = adminPassword.value.trim();
    const confirmPassword = adminPasswordConfirm.value.trim();

    if (!password) {
        setMessage('请输入管理口令。', 'error');
        return;
    }

    if (!confirmPassword) {
        setMessage('请再次输入管理口令。', 'error');
        return;
    }

    setupBtn.disabled = true;
    setMessage('正在初始化管理员口令...');

    try {
        const response = await fetch('/api/v1/auth/setup', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                password,
                confirm_password: confirmPassword,
            }),
        });
        const payload = await response.json();

        if (!response.ok) {
            throw new Error(payload.error || '初始化失败');
        }

        adminPassword.value = '';
        adminPasswordConfirm.value = '';
        setAdminState(Boolean(payload.authenticated), Boolean(payload.setup_required));
        await loadPhotos();
        setMessage('管理员口令已设置。', 'success');
    } catch (error) {
        setMessage(error.message || '初始化失败', 'error');
    } finally {
        setupBtn.disabled = false;
    }
});

logoutBtn?.addEventListener('click', async () => {
    logoutBtn.disabled = true;

    try {
        await fetch('/api/v1/auth/logout', { method: 'POST' });
    } finally {
        setAdminState(false);
        await loadPhotos();
        setMessage('已退出管理模式。');
        logoutBtn.disabled = false;
    }
});

galleryGrid?.addEventListener('click', async (event) => {
    const card = event.target.closest('[data-photo-id]');
    const blocked = event.target.closest('button, form, input, textarea');
    if (card && !blocked) {
        const photo = currentPhotos.find((item) => String(item.id) === card.dataset.photoId);
        if (photo) openPhotoModal(photo);
        return;
    }

    const editButton = event.target.closest('[data-edit-id]');
    if (editButton) {
        if (!isAdmin) {
            setMessage('请先登录管理口令。', 'error');
            return;
        }

        const { editId } = editButton.dataset;
        const formNode = galleryGrid.querySelector(`[data-editor-id="${editId}"]`);
        formNode?.classList.toggle('show');
        return;
    }

    const cancelButton = event.target.closest('[data-cancel-edit]');
    if (cancelButton) {
        const { cancelEdit } = cancelButton.dataset;
        const formNode = galleryGrid.querySelector(`[data-editor-id="${cancelEdit}"]`);
        formNode?.classList.remove('show');
        return;
    }

    const button = event.target.closest('[data-delete-id]');
    if (!button) return;

    if (!isAdmin) {
        setMessage('请先登录管理口令。', 'error');
        return;
    }

    const { deleteId } = button.dataset;
    if (!deleteId) return;

    if (!window.confirm('确定删除这张照片吗？')) {
        return;
    }

    button.disabled = true;

    try {
        const response = await fetch(`/api/v1/photos/${deleteId}`, { method: 'DELETE' });
        const payload = await response.json();

        if (!response.ok) {
            throw new Error(payload.error || '删除失败');
        }

        await loadPhotos();
        setMessage(`已删除照片 #${payload.id}`, 'success');
    } catch (error) {
        setMessage(error.message || '删除失败', 'error');
        button.disabled = false;
    }
});

galleryGrid?.addEventListener('submit', async (event) => {
    const editForm = event.target.closest('[data-editor-id]');
    if (!editForm) return;

    event.preventDefault();

    if (!isAdmin) {
        setMessage('请先登录管理口令。', 'error');
        return;
    }

    const photoId = editForm.dataset.editorId;
    const formData = new FormData(editForm);
    const submitNode = editForm.querySelector('button[type="submit"]');
    submitNode.disabled = true;

    try {
        const response = await fetch(`/api/v1/photos/${photoId}`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                title: formData.get('title'),
                description: formData.get('description'),
                tags: formData.get('tags'),
            }),
        });
        const payload = await response.json();

        if (!response.ok) {
            throw new Error(payload.error || '保存失败');
        }

        await loadPhotos();
        setMessage(`已更新照片：${payload.item.title}`, 'success');
    } catch (error) {
        setMessage(error.message || '保存失败', 'error');
    } finally {
        submitNode.disabled = false;
    }
});

photoModal?.addEventListener('click', (event) => {
    if (event.target.closest('[data-close-modal="true"]')) {
        closePhotoModal();
    }
});

modalCloseBtn?.addEventListener('click', closePhotoModal);
window.addEventListener('keydown', (event) => {
    if (event.key === 'Escape' && !photoModal.classList.contains('hidden')) {
        closePhotoModal();
    }
});

const reveal = () => {
    const io = new IntersectionObserver((entries) => {
        entries.forEach((entry) => {
            entry.target.classList.toggle('in', entry.isIntersecting);
        });
    }, { threshold: 0.12 });

    document.querySelectorAll('.reveal').forEach((el) => io.observe(el));
};

const syncAuth = async () => {
    const response = await fetch('/api/v1/auth/status');
    if (!response.ok) {
        throw new Error('读取鉴权状态失败');
    }

    const payload = await response.json();
    setAdminState(Boolean(payload.authenticated), Boolean(payload.setup_required));
};

(async () => {
    reveal();

    try {
        const response = await fetch('/api/v1/health');
        const payload = await response.json();
        if (payload.status === 'ok') {
            healthDot?.classList.add('ok');
        }
    } catch {
        healthDot?.classList.remove('ok');
    }

    try {
        await syncAuth();
    } catch {
        setAdminState(false, false);
    }

    try {
        await loadPhotos();
    } catch {
        emptyState.classList.remove('hidden');
    }
})();
