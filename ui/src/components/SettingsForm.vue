<template>
    <div>
        <div v-if="loading" class="card">Loading...</div>
        <div v-else>
            <div class="card">
                <div class="sec">Telegram</div>
                <div class="field"><label>Bot Token</label><input v-model="form.bot_token" placeholder="Enter new token to change"><div class="hint">Current: {{ config.bot_token_masked }}</div></div>
                <div class="field"><label>Authorized User IDs</label><input v-model="usersStr" placeholder="Comma-separated"><div class="hint">Only these Telegram users can talk to the bot</div></div>
            </div>
            <div class="card">
                <div class="sec">Download Destinations</div>
                <div class="hint-block">Configure named destinations. When sending a .torrent file, the bot will ask which destination to use.</div>
                <div class="field" v-for="(value, key) in form.destinations" :key="key">
                    <label>{{ key }}</label>
                    <div class="dest-row">
                        <input v-model="form.destinations[key]" placeholder="/volume1/video/movies">
                        <button v-if="key !== 'default'" class="btn-sm btn-del" @click="removeDest(key)">✕</button>
                    </div>
                </div>
                <div class="dest-add">
                    <input v-model="newKey" placeholder="Name (e.g. movies)" class="dest-input">
                    <input v-model="newVal" placeholder="Path (e.g. video/movies)" class="dest-input">
                    <button class="btn-sm btn-add" @click="addDest">+ Add</button>
                </div>
            </div>
            <div class="actions">
                <button class="btn" @click="save" :disabled="saving">{{ saving ? 'Saving...' : 'Save' }}</button>
                <span :class="['msg', saveOk ? 'ok' : 'err']" v-if="saveMsg">{{ saveMsg }}</span>
            </div>
            <div class="hint" style="margin-top:8px">After changing the bot token, restart the package from Package Center.</div>
        </div>
    </div>
</template>

<script>
import { getConfig, saveConfig } from '../api.js';
export default {
    data() {
        return { loading: true, saving: false, saveMsg: '', saveOk: false,
                 config: {}, form: { bot_token: '', destinations: {} }, usersStr: '',
                 newKey: '', newVal: '' };
    },
    mounted() { this.load(); },
    methods: {
        async load() {
            try {
                this.config = await getConfig();
                this.form.destinations = Object.assign({}, this.config.destinations || {});
                this.usersStr = (this.config.authorized_users || []).join(', ');
            } catch (e) { this.saveMsg = 'Failed to load'; this.saveOk = false; }
            this.loading = false;
        },
        addDest() {
            var k = this.newKey.trim().toLowerCase();
            var v = this.newVal.trim();
            if (k && v) { this.$set(this.form.destinations, k, v); this.newKey = ''; this.newVal = ''; }
        },
        removeDest(k) { this.$delete(this.form.destinations, k); },
        async save() {
            this.saving = true; this.saveMsg = '';
            var users = this.usersStr.split(',').map(function(s){return parseInt(s.trim())}).filter(function(n){return !isNaN(n)});
            var payload = { authorized_users: users, destinations: this.form.destinations };
            if (this.form.bot_token.trim()) payload.bot_token = this.form.bot_token.trim();
            try {
                // Try saving to multiple possible locations
                var saved = false;
                var folders = ['/watch', '/downloads'];
                for (var i = 0; i < folders.length && !saved; i++) {
                    try { await saveConfig(payload, folders[i]); saved = true; } catch(e) {}
                }
                if (!saved) throw new Error('Could not save to any shared folder');
                this.saveMsg = 'Saved!'; this.saveOk = true;
                this.form.bot_token = '';
                setTimeout(function() { this.load(); }.bind(this), 5000);
            } catch (e) { this.saveMsg = 'Save failed: ' + e.message; this.saveOk = false; }
            this.saving = false;
        }
    }
};
</script>

<style>
.card { background:#fff; border-radius:6px; padding:14px; box-shadow:0 1px 3px rgba(0,0,0,.06); margin-bottom:12px; }
.sec { font-size:12px; font-weight:600; color:#057FEB; text-transform:uppercase; letter-spacing:.4px; margin-bottom:10px; }
.field { margin-bottom:10px; }
.field label { display:block; font-size:12px; font-weight:500; color:#555; margin-bottom:3px; }
.field input { width:100%; padding:7px 9px; border:1px solid #ccc; border-radius:4px; font-size:13px; box-sizing:border-box; }
.field input:focus { border-color:#057FEB; outline:none; }
.hint { font-size:11px; color:#999; margin-top:2px; }
.hint-block { font-size:12px; color:#666; line-height:1.5; margin-bottom:10px; }
.dest-row { display:flex; gap:6px; align-items:center; }
.dest-row input { flex:1; }
.dest-add { display:flex; gap:6px; margin-top:8px; }
.dest-input { flex:1; padding:7px 9px; border:1px solid #ccc; border-radius:4px; font-size:13px; }
.btn-sm { padding:5px 10px; border:none; border-radius:4px; cursor:pointer; font-size:12px; }
.btn-del { background:#ff3b30; color:#fff; }
.btn-add { background:#e8f5e9; color:#2e7d32; }
.actions { display:flex; align-items:center; gap:10px; }
.btn { padding:8px 18px; border:none; border-radius:5px; cursor:pointer; font-size:13px; font-weight:500; background:#057FEB; color:#fff; }
.btn:hover { background:#046fd4; }
.btn:disabled { background:#a0c8f0; }
.msg { font-size:12px; }
.msg.ok { color:#34c759; }
.msg.err { color:#ff3b30; }
</style>
