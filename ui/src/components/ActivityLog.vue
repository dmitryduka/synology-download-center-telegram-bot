<template>
    <div class="activity-log">
        <h2>Activity Log</h2>
        <div v-if="entries.length === 0" class="empty-state">
            No activity recorded yet. Download events will appear here.
        </div>
        <div v-else class="log-list">
            <div v-for="(entry, idx) in entries" :key="idx" :class="['log-entry', 'level-' + entry.level]">
                <span class="log-time">{{ formatTime(entry.timestamp) }}</span>
                <span :class="['log-level', 'level-' + entry.level]">{{ entry.level }}</span>
                <span class="log-msg">{{ entry.message }}</span>
            </div>
        </div>
        <button class="btn-outline" @click="fetchLog" style="margin-top: 12px;">Refresh</button>
    </div>
</template>

<script>
import { getActivity } from '../api.js';

export default {
    data() { return { entries: [], timer: null }; },
    mounted() { this.fetchLog(); this.timer = setInterval(() => this.fetchLog(), 15000); },
    beforeDestroy() { if (this.timer) clearInterval(this.timer); },
    methods: {
        async fetchLog() {
            try { this.entries = await getActivity(); } catch (e) {}
        },
        formatTime(ts) {
            const d = new Date(parseInt(ts, 10) * 1000);
            return isNaN(d.getTime()) ? ts : d.toLocaleString();
        },
    },
};
</script>

<style>
.activity-log h2 { margin: 0 0 16px; font-size: 18px; font-weight: 500; }
.empty-state { text-align: center; padding: 40px; color: #999; font-size: 14px; background: #fff; border-radius: 8px; }
.log-list { background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.08); }
.log-entry { padding: 10px 14px; border-bottom: 1px solid #f0f0f0; display: flex; gap: 10px; align-items: baseline; font-size: 13px; }
.log-entry:last-child { border-bottom: none; }
.log-time { color: #999; font-size: 12px; white-space: nowrap; min-width: 140px; }
.log-level { font-size: 11px; font-weight: 600; text-transform: uppercase; padding: 2px 6px; border-radius: 3px; min-width: 50px; text-align: center; }
.log-level.level-info { background: #e3f2fd; color: #1565c0; }
.log-level.level-error { background: #ffebee; color: #c62828; }
.log-level.level-success { background: #e8f5e9; color: #2e7d32; }
.log-msg { flex: 1; }
.btn-outline { padding: 8px 16px; border: 1px solid #ccc; border-radius: 6px; background: #fff; cursor: pointer; font-size: 13px; }
.btn-outline:hover { background: #f5f5f5; }
</style>
