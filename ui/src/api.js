const DATA_BASE = '/webman/3rdparty/SynoTelegramBot/data';

export async function getStatus() {
    const resp = await fetch(DATA_BASE + '/status.json?_=' + Date.now());
    if (!resp.ok) throw new Error('Status unavailable');
    return resp.json();
}

export async function getConfig() {
    const resp = await fetch(DATA_BASE + '/config_read.json?_=' + Date.now());
    if (!resp.ok) throw new Error('Config unavailable');
    return resp.json();
}

export async function saveConfig(payload, watchFolder) {
    const blob = new Blob([JSON.stringify(payload)], { type: 'application/json' });
    const form = new FormData();
    form.append('api', 'SYNO.FileStation.Upload');
    form.append('version', '2');
    form.append('method', 'upload');
    form.append('path', watchFolder);
    form.append('create_parents', 'true');
    form.append('overwrite', 'true');
    form.append('file', blob, 'config_update.json');

    const resp = await fetch('/webapi/entry.cgi', { method: 'POST', body: form });
    const result = await resp.json();
    if (!result.success) {
        throw new Error('Upload failed (code ' + (result.error && result.error.code) + ')');
    }
    return result;
}
