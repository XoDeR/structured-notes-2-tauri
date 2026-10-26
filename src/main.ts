import '@fontsource-variable/instrument-sans';
import { createApp } from 'vue';
import App from '@/App.vue';
import { initializeTheme } from '@/composables/useAppearance';
import { router } from '@/router';
import './assets/app.css';

initializeTheme();

createApp(App)
    .use(router)
    .directive('focus', {
        mounted: (el: HTMLElement, shouldFocus) => {
            if (shouldFocus.value !== false) {
                el.focus();
            }
        },
    })
    .mount('#app');
