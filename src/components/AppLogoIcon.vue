<script setup lang="ts">
import { NotebookText } from '@lucide/vue';
import type { HTMLAttributes } from 'vue';

defineOptions({
    inheritAttrs: false,
});

type Props = {
    className?: HTMLAttributes['class'];
};

defineProps<Props>();
</script>

<template>
    <NotebookText :class="className" v-bind="$attrs" />
</template>
