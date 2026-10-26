import { createRouter, createWebHashHistory } from 'vue-router';
import type { BreadcrumbItem } from '@/types';

declare module 'vue-router' {
    interface RouteMeta {
        breadcrumbs?: BreadcrumbItem[];
    }
}

export const router = createRouter({
    history: createWebHashHistory(),
    routes: [
        { path: '/', redirect: '/notes' },
        {
            path: '/notes',
            name: 'notes',
            component: () => import('@/views/NotesView.vue'),
            meta: { breadcrumbs: [{ title: 'Notes', href: '/notes' }] },
        },
        { path: '/settings', redirect: '/settings/appearance' },
        {
            path: '/settings/appearance',
            name: 'appearance',
            component: () => import('@/views/AppearanceView.vue'),
            meta: {
                breadcrumbs: [
                    {
                        title: 'Appearance settings',
                        href: '/settings/appearance',
                    },
                ],
            },
        },
    ],
});
