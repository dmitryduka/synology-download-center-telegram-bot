const API_NAME = 'SYNO.SynoTelegramBot.API';
const API_VERSION = 1;

function webapiUrl(method) {
    return '/webapi/entry.cgi?api=' + API_NAME + '&version=' + API_VERSION + '&method=' + method;
}

export async function apiCall(method, body) {
    const url = webapiUrl(method);
    const opts = {};
    if (body) {
        opts.method = 'POST';
        opts.headers = { 'Content-Type': 'application/json' };
        opts.body = JSON.stringify(body);
    }
    const resp = await fetch(url, opts);
    const json = await resp.json();
    if (json.success && json.data) {
        return json.data;
    }
    throw new Error(json.error ? 'API error ' + json.error.code : 'Unknown error');
}

export async function getStatus() {
    return apiCall('get_status');
}

export async function getConfig() {
    return apiCall('get_config');
}

export async function setConfig(payload) {
    return apiCall('set_config', payload);
}

export async function getActivity() {
    return apiCall('get_activity');
}
