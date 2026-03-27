<template>
    <v-app-instance class-name="SYNO.SDS.App.SynoTelegramBot.Instance">
        <v-app-window
            width="850"
            height="574"
            ref="appWindow"
            :resizable="true"
            syno-id="SYNO.SDS.App.SynoTelegramBot.Window"
        >
            <div class="syno-tgbot-app">
                <div class="syno-tgbot-tabs">
                    <button
                        v-for="tab in tabs"
                        :key="tab.id"
                        :class="['syno-tgbot-tab', { active: activeTab === tab.id }]"
                        @click="activeTab = tab.id"
                    >
                        {{ tab.label }}
                    </button>
                </div>

                <div class="syno-tgbot-content">
                    <StatusPanel v-if="activeTab === 'status'" />
                    <SettingsForm v-if="activeTab === 'settings'" />
                    <ActivityLog v-if="activeTab === 'activity'" />
                </div>
            </div>
        </v-app-window>
    </v-app-instance>
</template>

<script>
import StatusPanel from './components/StatusPanel.vue';
import SettingsForm from './components/SettingsForm.vue';
import ActivityLog from './components/ActivityLog.vue';

export default {
    components: { StatusPanel, SettingsForm, ActivityLog },
    data() {
        return {
            activeTab: 'status',
            tabs: [
                { id: 'status', label: 'Status' },
                { id: 'settings', label: 'Settings' },
                { id: 'activity', label: 'Activity' },
            ],
        };
    },
};
</script>

<style>
.syno-tgbot-app {
    height: 100%;
    display: flex;
    flex-direction: column;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    color: #333;
    background: #f5f5f5;
}
.syno-tgbot-tabs {
    display: flex;
    background: #fff;
    border-bottom: 1px solid #ddd;
    padding: 0 16px;
}
.syno-tgbot-tab {
    padding: 12px 20px;
    border: none;
    background: none;
    cursor: pointer;
    font-size: 14px;
    color: #666;
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
}
.syno-tgbot-tab:hover { color: #333; }
.syno-tgbot-tab.active {
    color: #057FEB;
    border-bottom-color: #057FEB;
    font-weight: 500;
}
.syno-tgbot-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
}
</style>
