<template>
    <div class="settings-form">
        <h2>Settings</h2>
        <div v-if="loading" class="loading">Loading configuration...</div>
        <div v-else>
            <section class="settings-section">
                <h3>Telegram</h3>
                <div class="field-group">
                    <label>Bot Token</label>
                    <input type="text" v-model="form.bot_token" placeholder="Enter new token to change" />
                    <span class="field-hint">Current: {{ config.telegram.bot_token_masked }}</span>
                </div>
                <div class="field-group">
                    <label>Authorized User IDs</label>
                    <input type="text" v-model="authorizedUsersStr" placeholder="Comma-separated IDs" />
                    <span class="field-hint">Telegram user IDs allowed to interact with the bot</span>
                </div>
            </section>
            <section class="settings-section">
                <h3>Synology</h3>
                <div class="field-group">
                    <label>Runner (DSM User)</label>
                    <input type="text" v-model="form.runner" />
                    <span class="field-hint">DSM username for Download Station API access via synowebapi</span>
                </div>
            </section>
            <section class="settings-section">
                <h3>Download Destinations</h3>
                <div class="field-group" v-for="(value, key) in form.destinations" :key="key">
                    <label>{{ key }}</label>
                    <div class="dest-row">
                        <input type="text" v-model="form.destinations[key]" />
                        <button v-if="key !== 'default'" class="btn-small btn-danger" @click="removeDest(key)">&#x2715;</button>
                    </div>
                </div>
                <div class="dest-add">
                    <input type="text" v-model="newDestKey" placeholder="Alias name" class="dest-add-input" />
                    <input type="text" v-model="newDestValue" placeholder="Folder path" class="dest-add-input" />
                    <button class="btn-small btn-primary" @click="addDest">Add</button>
                </div>
            </section>
            <section class="settings-section">
                <h3>Notifications</h3>
                <div class="field-group">
                    <label>Poll Interval (seconds)</label>
                    <input type="number" v-model.number="form.poll_interval_secs" min="10" max="600" />
                </div>
            </section>
            <div class="form-actions">
                <button class="btn btn-primary" @click="save" :disabled="saving">
                    {{ saving ? 'Saving...' : 'Save Settings' }}
                </button>
                <span v-if="saveMsg" :class="['save-msg', saveOk ? 'ok' : 'err']">{{ saveMsg }}</span>
            </div>
        </div>
    </div>
</template>

<script>
import { getConfig, setConfig } from '../api.js';

export default {
    data() {
        return {
            loading: true, saving: false, saveMsg: '', saveOk: false,
            config: { telegram: {}, synology: {}, destinations: {}, notifications: {} },
            form: { bot_token: '', runner: '', destinations: {}, poll_interval_secs: 30 },
            authorizedUsersStr: '', newDestKey: '', newDestValue: '',
        };
    },
    mounted() { this.fetchConfig(); },
    methods: {
        async fetchConfig() {
            try {
                this.config = await getConfig();
                this.form.runner = this.config.synology.runner;
                this.form.destinations = { ...this.config.destinations };
                this.form.poll_interval_secs = this.config.notifications.poll_interval_secs;
                this.authorizedUsersStr = (this.config.telegram.authorized_users || []).join(', ');
            } catch (e) { this.saveMsg = 'Failed to load config'; this.saveOk = false; }
            this.loading = false;
        },
        addDest() {
            const key = this.newDestKey.trim().toLowerCase();
            const value = this.newDestValue.trim();
            if (key && value) { this.$set(this.form.destinations, key, value); this.newDestKey = ''; this.newDestValue = ''; }
        },
        removeDest(key) { this.$delete(this.form.destinations, key); },
        async save() {
            this.saving = true; this.saveMsg = '';
            const users = this.authorizedUsersStr.split(',').map(s => parseInt(s.trim(), 10)).filter(n => !isNaN(n));
            const payload = {
                telegram: { authorized_users: users },
                synology: { runner: this.form.runner },
                destinations: this.form.destinations,
                notifications: { poll_interval_secs: this.form.poll_interval_secs },
            };
            if (this.form.bot_token.trim()) payload.telegram.bot_token = this.form.bot_token.trim();
            try {
                await setConfig(payload);
                this.saveMsg = 'Settings saved. Restart the package for token changes to take effect.';
                this.saveOk = true; this.form.bot_token = ''; this.fetchConfig();
            } catch (e) { this.saveMsg = 'Save failed: ' + e.message; this.saveOk = false; }
            this.saving = false;
        },
    },
};
</script>

<style>
.settings-form h2 { margin: 0 0 16px; font-size: 18px; font-weight: 500; }
.settings-section { background: #fff; border-radius: 8px; padding: 16px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.settings-section h3 { margin: 0 0 12px; font-size: 14px; font-weight: 600; color: #057FEB; text-transform: uppercase; letter-spacing: 0.5px; }
.field-group { margin-bottom: 12px; }
.field-group label { display: block; font-size: 13px; font-weight: 500; margin-bottom: 4px; color: #555; }
.field-group input { width: 100%; padding: 8px 10px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; box-sizing: border-box; }
.field-group input:focus { border-color: #057FEB; outline: none; }
.field-hint { font-size: 11px; color: #999; margin-top: 2px; display: block; }
.dest-row { display: flex; gap: 8px; align-items: center; }
.dest-row input { flex: 1; }
.dest-add { display: flex; gap: 8px; margin-top: 8px; }
.dest-add-input { flex: 1; padding: 8px 10px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; }
.btn { padding: 10px 24px; border: none; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: 500; }
.btn-primary { background: #057FEB; color: #fff; }
.btn-primary:hover { background: #046fd4; }
.btn-primary:disabled { background: #a0c8f0; cursor: default; }
.btn-small { padding: 6px 12px; border: none; border-radius: 4px; cursor: pointer; font-size: 12px; }
.btn-danger { background: #ff3b30; color: #fff; }
.form-actions { display: flex; align-items: center; gap: 12px; margin-top: 8px; }
.save-msg { font-size: 13px; }
.save-msg.ok { color: #34c759; }
.save-msg.err { color: #ff3b30; }
</style>
