<template>
    <div class="status-panel">
        <h2>Bot Status</h2>
        <div class="status-grid">
            <div class="status-card">
                <div :class="['status-indicator', status.bot_running ? 'green' : 'red']"></div>
                <div class="status-info">
                    <div class="status-label">Telegram Bot</div>
                    <div class="status-value">{{ status.bot_running ? 'Running' : 'Stopped' }}</div>
                </div>
            </div>
            <div class="status-card">
                <div :class="['status-indicator', connected ? 'green' : 'red']"></div>
                <div class="status-info">
                    <div class="status-label">API Connection</div>
                    <div class="status-value">{{ connected ? 'Connected' : 'Unreachable' }}</div>
                </div>
            </div>
            <div class="status-card">
                <div class="status-info">
                    <div class="status-label">Version</div>
                    <div class="status-value">{{ status.version || '—' }}</div>
                </div>
            </div>
        </div>
        <div v-if="error" class="error-banner">{{ error }}</div>
    </div>
</template>

<script>
import { getStatus } from '../api.js';

export default {
    data() {
        return {
            status: { bot_running: false, version: '' },
            connected: false,
            error: null,
            timer: null,
        };
    },
    mounted() {
        this.fetchStatus();
        this.timer = setInterval(() => this.fetchStatus(), 10000);
    },
    beforeDestroy() {
        if (this.timer) clearInterval(this.timer);
    },
    methods: {
        async fetchStatus() {
            try {
                this.status = await getStatus();
                this.connected = true;
                this.error = null;
            } catch (e) {
                this.status = { bot_running: false, version: '' };
                this.connected = false;
                this.error = 'Cannot reach the bot API. Is the package running?';
            }
        },
    },
};
</script>

<style>
.status-panel h2 { margin: 0 0 16px; font-size: 18px; font-weight: 500; }
.status-grid { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 12px; }
.status-card { background: #fff; border-radius: 8px; padding: 16px; display: flex; align-items: center; gap: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.status-indicator { width: 12px; height: 12px; border-radius: 50%; flex-shrink: 0; }
.status-indicator.green { background: #34c759; box-shadow: 0 0 6px rgba(52,199,89,0.4); }
.status-indicator.red { background: #ff3b30; box-shadow: 0 0 6px rgba(255,59,48,0.4); }
.status-label { font-size: 12px; color: #888; margin-bottom: 2px; }
.status-value { font-size: 16px; font-weight: 500; }
.error-banner { margin-top: 16px; padding: 12px; background: #fff3f3; border: 1px solid #ffcdd2; border-radius: 6px; color: #c62828; font-size: 13px; }
</style>
