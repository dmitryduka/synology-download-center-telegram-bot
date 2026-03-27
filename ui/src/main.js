import Vue from 'vue';
import App from './App.vue';

SYNO.namespace('SYNO.SDS.TelegramBot');

SYNO.SDS.TelegramBot.Instance = Vue.extend({
    components: { App },
    template: '<App />',
});
