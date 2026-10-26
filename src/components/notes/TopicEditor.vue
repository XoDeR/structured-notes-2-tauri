<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog';
import { Paperclip, Upload, X } from '@lucide/vue';
import { computed, ref } from 'vue';
import { toast } from 'vue-sonner';
import RichTextEditor from '@/components/notes/RichTextEditor.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useNotes } from '@/composables/useNotes';
import { api, errorMessage } from '@/lib/api';
import type { TopicDetail } from '@/types/notes';

const props = defineProps<{
    /** The topic being edited, or null to create a new one in `nodeId`. */
    topic: TopicDetail | null;
    nodeId: string;
    knownTags: string[];
}>();

const emit = defineEmits<{ close: [] }>();

const { saveTopic, refreshTopic } = useNotes();

const title = ref(props.topic?.title ?? '');
const content = ref(props.topic?.content ?? '');
const tags = ref([...(props.topic?.tags ?? [])]);

const tagDraft = ref('');
const processing = ref(false);
const uploading = ref(false);

const suggestions = computed(() =>
    props.knownTags.filter((t) => !tags.value.includes(t)),
);

function addTag(): void {
    const value = tagDraft.value.trim().replace(/[;,]+$/, '');

    if (value && !tags.value.includes(value)) {
        tags.value.push(value);
    }

    tagDraft.value = '';
}

function onTagKey(event: KeyboardEvent): void {
    if (event.key === 'Enter' || event.key === ',' || event.key === ';') {
        event.preventDefault();
        addTag();
    } else if (event.key === 'Backspace' && !tagDraft.value) {
        tags.value.pop();
    }
}

async function submit(): Promise<void> {
    addTag();
    processing.value = true;

    const saved = await saveTopic(props.topic?.id ?? null, props.nodeId, {
        title: title.value,
        content: content.value,
        tags: tags.value,
    });

    processing.value = false;

    if (saved) {
        emit('close');
    }
}

async function pickFile(): Promise<void> {
    if (!props.topic) {
        return;
    }

    const path = await open({ multiple: false, directory: false });

    if (!path) {
        return;
    }

    uploading.value = true;

    try {
        await api.addAttachment(props.topic.id, path);
        await refreshTopic();
    } catch (error) {
        toast.error(errorMessage(error));
    } finally {
        uploading.value = false;
    }
}

async function removeFile(id: string): Promise<void> {
    try {
        await api.deleteMedia(id);
        await refreshTopic();
    } catch (error) {
        toast.error(errorMessage(error));
    }
}
</script>

<template>
    <form class="space-y-4" @submit.prevent="submit">
        <div class="grid gap-2">
            <Label for="topic-title">Title</Label>
            <Input
                id="topic-title"
                v-model="title"
                maxlength="255"
                required
                autofocus
            />
        </div>

        <RichTextEditor v-model="content" :topic-id="topic?.id ?? null" />

        <div class="grid gap-2">
            <Label for="topic-tags">Tags</Label>
            <div
                class="flex flex-wrap items-center gap-1.5 rounded-md border px-2 py-1.5"
            >
                <Badge
                    v-for="tag in tags"
                    :key="tag"
                    variant="secondary"
                    class="gap-1"
                >
                    {{ tag }}
                    <button
                        type="button"
                        :aria-label="`Remove ${tag}`"
                        @click="tags = tags.filter((t) => t !== tag)"
                    >
                        <X class="size-3" />
                    </button>
                </Badge>
                <input
                    id="topic-tags"
                    v-model="tagDraft"
                    list="known-tags"
                    class="min-w-32 flex-1 bg-transparent text-sm outline-none"
                    placeholder="Add a tag and press Enter"
                    @keydown="onTagKey"
                    @blur="addTag"
                />
                <datalist id="known-tags">
                    <option
                        v-for="tag in suggestions"
                        :key="tag"
                        :value="tag"
                    />
                </datalist>
            </div>
        </div>

        <div class="grid gap-2">
            <Label>Files</Label>
            <div v-if="topic" class="flex flex-wrap items-center gap-2">
                <span
                    v-for="file in topic.attachments"
                    :key="file.id"
                    class="inline-flex items-center gap-1 rounded-md border px-2 py-1 text-sm"
                >
                    <Paperclip class="size-3.5" />
                    {{ file.name }}
                    <button
                        type="button"
                        :aria-label="`Delete ${file.name}`"
                        class="text-destructive"
                        @click="removeFile(file.id)"
                    >
                        <X class="size-3.5" />
                    </button>
                </span>
                <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    :disabled="uploading"
                    @click="pickFile"
                >
                    <Upload /> Add file
                </Button>
            </div>
            <p v-else class="text-sm text-muted-foreground">
                Save the topic first, then you can attach files and add images.
            </p>
        </div>

        <div class="flex justify-end gap-2">
            <Button type="button" variant="outline" @click="emit('close')"
                >Cancel</Button
            >
            <Button type="submit" :disabled="processing">Save</Button>
        </div>
    </form>
</template>
