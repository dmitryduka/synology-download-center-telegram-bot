import Vue from 'vue';
import App from './App.vue';

SYNO.namespace('SYNO.SDS.App.SynoTelegramBot');

SYNO.SDS.App.SynoTelegramBot.Instance = Vue.extend({
    components: { App },
    template: '<App />',
});
