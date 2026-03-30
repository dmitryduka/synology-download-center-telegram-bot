const WEBAPI = '/webapi/entry.cgi';
const DATA_BASE = '/webman/3rdparty/SynoTelegramBot/data';
const SERVICE_USER = 'tgbot-service';

function getSynoToken() {
    if (typeof _S === 'function') {
        try { return _S('SynoToken'); } catch(e) {}
    }
    return '';
}

async function synoApi(api, version, method, params) {
    var url = new URL(WEBAPI, window.location.origin);
    url.searchParams.set('api', api);
    url.searchParams.set('version', version);
    url.searchParams.set('method', method);
    var token = getSynoToken();
    if (token) url.searchParams.set('SynoToken', token);
    for (var k in params) {
        if (params.hasOwnProperty(k)) {
            var v = params[k];
            url.searchParams.set(k, typeof v === 'object' ? JSON.stringify(v) : v);
        }
    }
    var resp = await fetch(url.toString(), { credentials: 'same-origin' });
    var data = await resp.json();
    if (!data.success) throw new Error('SYNO API error ' + (data.error && data.error.code));
    return data.data;
}

// --- tmpfs reads ---

export async function getStatus() {
    var resp = await fetch(DATA_BASE + '/status.json?_=' + Date.now(), { credentials: 'same-origin' });
    if (!resp.ok) throw new Error('Status unavailable');
    return resp.json();
}

export async function getConfig() {
    var resp = await fetch(DATA_BASE + '/config_read.json?_=' + Date.now(), { credentials: 'same-origin' });
    if (!resp.ok) throw new Error('Config unavailable');
    return resp.json();
}

// --- FileStation write ---

function toFileStationPath(path) {
    // FileStation uses /sharename not /volume1/sharename
    return path.replace(/^\/volume\d+/, '');
}

async function uploadFile(watchFolder, filename, content) {
    var blob = new Blob([content], { type: 'application/json' });
    var form = new FormData();
    form.append('api', 'SYNO.FileStation.Upload');
    form.append('version', '2');
    form.append('method', 'upload');
    form.append('path', toFileStationPath(watchFolder));
    form.append('create_parents', 'true');
    form.append('overwrite', 'true');
    form.append('file', blob, filename);
    var url = WEBAPI;
    var token = getSynoToken();
    if (token) url += '?SynoToken=' + encodeURIComponent(token);
    var resp = await fetch(url, { method: 'POST', body: form, credentials: 'same-origin' });
    var result = await resp.json();
    if (!result.success) throw new Error('Upload failed (code ' + (result.error && result.error.code) + ')');
}

export async function saveConfig(payload, watchFolder) {
    await uploadFile(watchFolder, 'config_update.json', JSON.stringify(payload));
}

// --- Service user setup ---

function randomPassword(len) {
    var chars = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%';
    var pw = '';
    for (var i = 0; i < len; i++) pw += chars.charAt(Math.floor(Math.random() * chars.length));
    return pw;
}

export async function setupBotAccess(watchFolder) {
    var password = randomPassword(24);

    // Create the service user (or update password if exists)
    try {
        await synoApi('SYNO.Core.User', 1, 'create', {
            name: SERVICE_USER,
            password: password,
            description: 'Telegram Download Bot service account',
            email: '',
            cannot_chg_passwd: true,
        });
    } catch(e) {
        // User may already exist — try setting password
        try {
            await synoApi('SYNO.Core.User', 1, 'set', {
                name: SERVICE_USER,
                password: password,
            });
        } catch(e2) {
            throw new Error('Failed to create/update service user: ' + e2.message);
        }
    }

    // Add to administrators group (needed to see all users' downloads)
    try {
        await synoApi('SYNO.Core.Group.Member', 1, 'add', {
            group: 'administrators',
            member: JSON.stringify([SERVICE_USER]),
        });
    } catch(e) {
        // May fail — user can add manually via Control Panel > User & Group
    }

    // Grant Download Station access
    try {
        await synoApi('SYNO.Core.AppPriv', 2, 'set', {
            name: SERVICE_USER,
            priv: JSON.stringify([{ app: 'SYNO.SDS.DownloadStation.Application', permission: 'allow' }]),
        });
    } catch(e) {
        // Non-critical
    }

    // Save credentials for the bot
    var creds = { username: SERVICE_USER, password: password };
    await uploadFile(watchFolder, 'service_credentials.json', JSON.stringify(creds));

    return SERVICE_USER;
}
