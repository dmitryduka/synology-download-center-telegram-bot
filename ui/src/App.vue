<template>
    <v-app-instance class-name="SYNO.SDS.TelegramBot.Instance">
        <v-app-window
            width="700"
            height="520"
            ref="appWindow"
            :resizable="true"
            syno-id="SYNO.SDS.TelegramBot.Window"
        >
            <div class="tgbot-app">
                <div class="tgbot-tabs">
                    <button v-for="tab in tabs" :key="tab.id"
                        :class="['tgbot-tab', { active: activeTab === tab.id }]"
                        @click="activeTab = tab.id">{{ tab.label }}</button>
                </div>
                <div class="tgbot-body">
                    <StatusPanel v-if="activeTab === 'status'" />
                    <SettingsForm v-if="activeTab === 'settings'" />
                </div>
            </div>
        </v-app-window>
    </v-app-instance>
</template>

<script>
import StatusPanel from './components/StatusPanel.vue';
import SettingsForm from './components/SettingsForm.vue';

export default {
    components: { StatusPanel, SettingsForm },
    data() {
        return {
            activeTab: 'status',
            tabs: [
                { id: 'status', label: 'Status' },
                { id: 'settings', label: 'Settings' },
            ],
        };
    },
};
</script>

<style>
.tgbot-app { height:100%; display:flex; flex-direction:column; font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif; color:#333; background:#f5f5f5; }
.tgbot-tabs { display:flex; background:#fff; border-bottom:1px solid #ddd; padding:0 16px; }
.tgbot-tab { padding:10px 20px; border:none; background:none; cursor:pointer; font-size:13px; color:#666; border-bottom:2px solid transparent; }
.tgbot-tab:hover { color:#333; }
.tgbot-tab.active { color:#057FEB; border-bottom-color:#057FEB; font-weight:500; }
.tgbot-body { flex:1; overflow-y:auto; padding:16px; }
</style>
