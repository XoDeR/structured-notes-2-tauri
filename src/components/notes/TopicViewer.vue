<script setup lang="ts">
import { save } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Paperclip, Tag } from '@lucide/vue';
import { computed } from 'vue';
import { toast } from 'vue-sonner';
import { Badge } from '@/components/ui/badge';
import { api, errorMessage, toDisplayHtml } from '@/lib/api';
import type { Attachment, TopicDetail } from '@/types/notes';

const props = defineProps<{ topic: TopicDetail }>();

const emit = defineEmits<{ tag: [tag: string] }>();

// Content is sanitized by the backend before it is stored.
const html = computed(() => toDisplayHtml(props.topic.content));

function formatSize(bytes: number): string {
    if (bytes < 1024) {
        return `${bytes} B`;
    }

    return bytes < 1024 * 1024
        ? `${(bytes / 1024).toFixed(1)} KB`
        : `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

/** Links inside a topic open in the system browser instead of replacing the app. */
function onContentClick(event: MouseEvent): void {
    const anchor = (event.target as HTMLElement).closest('a');

    if (!anchor) {
        return;
    }

    event.preventDefault();
    const href = anchor.getAttribute('href');

    if (href) {
        void openUrl(href).catch((error) => toast.error(errorMessage(error)));
    }
}

async function download(file: Attachment): Promise<void> {
    try {
        const path = await save({ defaultPath: file.name });

        if (path) {
            await api.exportMedia(file.id, path);
            toast.success('File saved.');
        }
    } catch (error) {
        toast.error(errorMessage(error));
    }
}
</script>

<template>
    <article class="space-y-4">
        <header class="space-y-1">
            <h2 class="text-xl font-semibold">{{ topic.title }}</h2>
            <p class="text-xs text-muted-foreground">
                {{ new Date(topic.updated_at).toLocaleString() }}
                <Badge v-if="topic.is_link" variant="secondary" class="ml-1"
                    >Link</Badge
                >
            </p>
        </header>

        <!-- eslint-disable-next-line vue/no-v-html -->
        <div
            class="prose prose-sm max-w-none dark:prose-invert"
            @click="onContentClick"
            v-html="html"
        />

        <footer
            v-if="topic.tags.length || topic.attachments.length"
            class="space-y-2 border-t pt-3 text-sm"
        >
            <div
                v-if="topic.tags.length"
                class="flex flex-wrap items-center gap-1.5"
            >
                <Tag class="size-4 text-muted-foreground" />
                <button
                    v-for="tag in topic.tags"
                    :key="tag"
                    type="button"
                    @click="emit('tag', tag)"
                >
                    <Badge
                        variant="secondary"
                        class="cursor-pointer hover:bg-accent"
                        >{{ tag }}</Badge
                    >
                </button>
            </div>
            <div
                v-if="topic.attachments.length"
                class="flex flex-wrap items-center gap-x-3 gap-y-1"
            >
                <Paperclip class="size-4 text-muted-foreground" />
                <button
                    v-for="file in topic.attachments"
                    :key="file.id"
                    type="button"
                    class="text-primary underline-offset-4 hover:underline"
                    @click="download(file)"
                >
                    {{ file.name }}
                    <span class="text-xs text-muted-foreground"
                        >({{ formatSize(file.size) }})</span
                    >
                </button>
            </div>
        </footer>
    </article>
</template>
