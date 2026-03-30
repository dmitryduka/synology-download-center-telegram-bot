<template>
    <div>
        <div class="card">
            <div class="row"><div :class="['dot', connected ? 'green' : 'red']"></div><span class="lbl">Bot</span><span class="val">{{ connected ? 'Running' : 'Stopped' }}</span></div>
            <div class="row"><span class="lbl">Version</span><span class="val">{{ status.version || '—' }}</span></div>
            <div class="row"><span class="lbl">Watch Folder</span><span class="val">{{ status.watch_folder || '—' }}</span></div>
            <div class="row"><span class="lbl">Download Station</span><span class="val">{{ status.ds_connected ? '✅ Connected' : '❌ Not configured' }}</span></div>
        </div>

        <div class="card" v-if="!status.ds_connected || showSetup">
            <div class="sec">Download Station Access</div>
            <p class="hint-block">
                The bot needs a service account to interact with Download Station
                (check status, send notifications, pause/delete downloads).
                Click below to create one automatically.
            </p>
            <div class="setup-row">
                <button class="btn" @click="setup" :disabled="setting">{{ setting ? 'Setting up...' : 'Setup Bot Access' }}</button>
                <span :class="['msg', setupOk ? 'ok' : 'err']" v-if="setupMsg">{{ setupMsg }}</span>
            </div>
        </div>

        <div v-if="error" class="err-box">{{ error }}</div>
    </div>
</template>

<script>
import { getStatus, setupBotAccess } from '../api.js';
export default {
    data() { return { status: {}, connected: false, error: null, timer: null, setting: false, setupMsg: '', setupOk: false, showSetup: false }; },
    mounted() { this.poll(); this.timer = setInterval(() => this.poll(), 8000); },
    beforeDestroy() { clearInterval(this.timer); },
    methods: {
        async poll() {
            try { this.status = await getStatus(); this.connected = true; this.error = null; }
            catch (e) { this.connected = false; this.error = 'Cannot read bot status. Is the package running?'; }
        },
        async setup() {
            this.setting = true; this.setupMsg = '';
            try {
                var user = await setupBotAccess(this.status.watch_folder);
                this.setupMsg = 'Created service account "' + user + '". The bot will pick it up shortly.';
                this.setupOk = true;
            } catch(e) {
                this.setupMsg = 'Failed: ' + e.message;
                this.setupOk = false;
            }
            this.setting = false;
        }
    }
};
</script>

<style>
.card { background:#fff; border-radius:6px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); margin-bottom:12px; }
.sec { font-size:12px; font-weight:600; color:#057FEB; text-transform:uppercase; letter-spacing:.4px; margin-bottom:8px; }
.row { display:flex; align-items:center; gap:8px; padding:6px 0; }
.dot { width:10px; height:10px; border-radius:50%; }
.dot.green { background:#34c759; box-shadow:0 0 5px rgba(52,199,89,.4); }
.dot.red { background:#ff3b30; box-shadow:0 0 5px rgba(255,59,48,.4); }
.lbl { font-size:12px; color:#888; }
.val { margin-left:auto; font-size:13px; font-weight:500; }
.err-box { padding:10px; background:#fff3f3; border:1px solid #ffcdd2; border-radius:6px; color:#c62828; font-size:12px; margin-top:8px; }
.hint-block { font-size:12px; color:#666; line-height:1.5; margin-bottom:10px; }
.setup-row { display:flex; align-items:center; gap:10px; }
.btn { padding:8px 18px; border:none; border-radius:5px; cursor:pointer; font-size:13px; font-weight:500; background:#057FEB; color:#fff; }
.btn:hover { background:#046fd4; }
.btn:disabled { background:#a0c8f0; }
.msg { font-size:12px; }
.msg.ok { color:#34c759; }
.msg.err { color:#ff3b30; }
</style>
