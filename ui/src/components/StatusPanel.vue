<template>
    <div>
        <div class="card">
            <div class="row"><div :class="['dot', connected ? 'green' : 'red']"></div><span class="lbl">Bot</span><span class="val">{{ connected ? 'Running' : 'Stopped' }}</span></div>
            <div class="row"><span class="lbl">Version</span><span class="val">{{ status.version || '—' }}</span></div>
            <div class="row"><span class="lbl">Watch Folder</span><span class="val">{{ status.watch_folder || '—' }}</span></div>
        </div>
        <div v-if="error" class="err">{{ error }}</div>
    </div>
</template>

<script>
import { getStatus } from '../api.js';
export default {
    data() { return { status: {}, connected: false, error: null, timer: null }; },
    mounted() { this.poll(); this.timer = setInterval(() => this.poll(), 8000); },
    beforeDestroy() { clearInterval(this.timer); },
    methods: {
        async poll() {
            try { this.status = await getStatus(); this.connected = true; this.error = null; }
            catch (e) { this.connected = false; this.error = 'Cannot read bot status. Is the package running?'; }
        }
    }
};
</script>

<style>
.card { background:#fff; border-radius:6px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); margin-bottom:12px; }
.row { display:flex; align-items:center; gap:8px; padding:6px 0; }
.dot { width:10px; height:10px; border-radius:50%; }
.dot.green { background:#34c759; box-shadow:0 0 5px rgba(52,199,89,.4); }
.dot.red { background:#ff3b30; box-shadow:0 0 5px rgba(255,59,48,.4); }
.lbl { font-size:12px; color:#888; }
.val { margin-left:auto; font-size:13px; font-weight:500; }
.err { padding:10px; background:#fff3f3; border:1px solid #ffcdd2; border-radius:6px; color:#c62828; font-size:12px; margin-top:8px; }
</style>
