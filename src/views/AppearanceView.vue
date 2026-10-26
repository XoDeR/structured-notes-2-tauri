<script setup lang="ts">
import AppearanceTabs from '@/components/AppearanceTabs.vue';
import Heading from '@/components/Heading.vue';
import SettingsLayout from '@/layouts/SettingsLayout.vue';
</script>

<template>
    <SettingsLayout>
        <h1 class="sr-only">Appearance settings</h1>

        <div class="space-y-6">
            <Heading
                variant="small"
                title="Appearance settings"
                description="Choose between a light, a dark or your system theme"
            />
            <AppearanceTabs />
        </div>
    </SettingsLayout>
</template>
