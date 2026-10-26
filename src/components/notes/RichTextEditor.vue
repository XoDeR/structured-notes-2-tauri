<script setup lang="ts">
import { Image } from '@tiptap/extension-image';
import { Placeholder } from '@tiptap/extension-placeholder';
import { StarterKit } from '@tiptap/starter-kit';
import { EditorContent, useEditor } from '@tiptap/vue-3';
import {
    Bold,
    Code,
    Code2,
    Heading2,
    Heading3,
    ImagePlus,
    Italic,
    Link2,
    Link2Off,
    List,
    ListOrdered,
    Minus,
    Quote,
    Redo2,
    Strikethrough,
    Underline,
    Undo2,
} from '@lucide/vue';
import { onBeforeUnmount, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import { Button } from '@/components/ui/button';
import { Toggle } from '@/components/ui/toggle';
import { api, errorMessage, mediaUrl, toDisplayHtml, toStoredHtml } from '@/lib/api';

const props = defineProps<{
    modelValue: string;
    /** Topic that owns embedded images; images are disabled until the topic is saved. */
    topicId?: string | null;
}>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const fileInput = ref<HTMLInputElement | null>(null);

const editor = useEditor({
    content: toDisplayHtml(props.modelValue),
    extensions: [
        StarterKit.configure({ link: { openOnClick: false } }),
        Image,
        Placeholder.configure({ placeholder: 'Write something…' }),
    ],
    editorProps: {
        attributes: {
            class: 'prose prose-sm dark:prose-invert max-w-none min-h-64 px-4 py-3 focus:outline-none',
        },
        handlePaste: (_view, event) => handleFiles(event.clipboardData?.files),
        handleDrop: (_view, event) => handleFiles(event.dataTransfer?.files),
    },
    onUpdate: ({ editor }) => emit('update:modelValue', toStoredHtml(editor.getHTML())),
});

watch(
    () => props.modelValue,
    (value) => {
        if (editor.value && value !== toStoredHtml(editor.value.getHTML())) {
            editor.value.commands.setContent(toDisplayHtml(value), {
                emitUpdate: false,
            });
        }
    },
);

onBeforeUnmount(() => editor.value?.destroy());

async function uploadImage(file: File): Promise<void> {
    if (!props.topicId) {
        return;
    }

    try {
        const id = await api.addImage(props.topicId, file);
        editor.value?.chain().focus().setImage({ src: mediaUrl(id) }).run();
    } catch (error) {
        toast.error(errorMessage(error));
    }
}
function handleFiles(files?: FileList | null): boolean {
    const images = Array.from(files ?? []).filter((f) =>
        f.type.startsWith('image/'),
    );

    if (!images.length) {
        return false;
    }

    if (!props.topicId) {
        toast.info('Save the topic first, then you can add images.');

        return true;
    }

    images.forEach((file) => void uploadImage(file));

    return true;
}

function onPickImage(event: Event): void {
    const input = event.target as HTMLInputElement;
    handleFiles(input.files);
    input.value = '';
}

function setLink(): void {
    const current = editor.value?.getAttributes('link').href ?? '';
    const url = window.prompt('Link address', current);

    if (url === null) {
        return;
    }

    if (url === '') {
        editor.value?.chain().focus().extendMarkRange('link').unsetLink().run();

        return;
    }

    editor.value
        ?.chain()
        .focus()
        .extendMarkRange('link')
        .setLink({ href: url })
        .run();
}
</script>

<template>
    <div class="overflow-hidden rounded-md border bg-background">
        <div
            v-if="editor"
            class="flex flex-wrap items-center gap-0.5 border-b bg-muted/40 p-1"
        >
            <Toggle
                size="sm"
                title="Bold"
                :model-value="editor.isActive('bold')"
                @click="editor.chain().focus().toggleBold().run()"
                ><Bold
            /></Toggle>
            <Toggle
                size="sm"
                title="Italic"
                :model-value="editor.isActive('italic')"
                @click="editor.chain().focus().toggleItalic().run()"
                ><Italic
            /></Toggle>
            <Toggle
                size="sm"
                title="Underline"
                :model-value="editor.isActive('underline')"
                @click="editor.chain().focus().toggleUnderline().run()"
                ><Underline
            /></Toggle>
            <Toggle
                size="sm"
                title="Strikethrough"
                :model-value="editor.isActive('strike')"
                @click="editor.chain().focus().toggleStrike().run()"
                ><Strikethrough
            /></Toggle>
            <Toggle
                size="sm"
                title="Inline code"
                :model-value="editor.isActive('code')"
                @click="editor.chain().focus().toggleCode().run()"
                ><Code
            /></Toggle>
            <span class="mx-1 h-5 w-px bg-border" />
            <Toggle
                size="sm"
                title="Heading"
                :model-value="editor.isActive('heading', { level: 2 })"
                @click="
                    editor.chain().focus().toggleHeading({ level: 2 }).run()
                "
                ><Heading2
            /></Toggle>
            <Toggle
                size="sm"
                title="Subheading"
                :model-value="editor.isActive('heading', { level: 3 })"
                @click="
                    editor.chain().focus().toggleHeading({ level: 3 }).run()
                "
                ><Heading3
            /></Toggle>
            <Toggle
                size="sm"
                title="Bullet list"
                :model-value="editor.isActive('bulletList')"
                @click="editor.chain().focus().toggleBulletList().run()"
                ><List
            /></Toggle>
            <Toggle
                size="sm"
                title="Numbered list"
                :model-value="editor.isActive('orderedList')"
                @click="editor.chain().focus().toggleOrderedList().run()"
                ><ListOrdered
            /></Toggle>
            <Toggle
                size="sm"
                title="Quote"
                :model-value="editor.isActive('blockquote')"
                @click="editor.chain().focus().toggleBlockquote().run()"
                ><Quote
            /></Toggle>
            <Toggle
                size="sm"
                title="Code block"
                :model-value="editor.isActive('codeBlock')"
                @click="editor.chain().focus().toggleCodeBlock().run()"
                ><Code2
            /></Toggle>
            <Button
                type="button"
                size="icon"
                variant="ghost"
                class="size-8"
                title="Horizontal rule"
                @click="editor.chain().focus().setHorizontalRule().run()"
                ><Minus
            /></Button>
            <span class="mx-1 h-5 w-px bg-border" />
            <Toggle
                size="sm"
                title="Link"
                :model-value="editor.isActive('link')"
                @click="setLink"
                ><Link2
            /></Toggle>
            <Button
                v-if="editor.isActive('link')"
                type="button"
                size="icon"
                variant="ghost"
                class="size-8"
                title="Remove link"
                @click="editor.chain().focus().unsetLink().run()"
                ><Link2Off
            /></Button>
            <Button
                type="button"
                size="icon"
                variant="ghost"
                class="size-8"
                :title="
                    topicId
                        ? 'Insert image'
                        : 'Save the topic first to add images'
                "
                :disabled="!topicId"
                @click="fileInput?.click()"
            >
                <ImagePlus />
            </Button>
            <input
                ref="fileInput"
                type="file"
                accept="image/png,image/jpeg,image/gif,image/webp"
                class="hidden"
                @change="onPickImage"
            />
            <span class="mx-1 h-5 w-px bg-border" />
            <Button
                type="button"
                size="icon"
                variant="ghost"
                class="size-8"
                title="Undo"
                :disabled="!editor.can().undo()"
                @click="editor.chain().focus().undo().run()"
                ><Undo2
            /></Button>
            <Button
                type="button"
                size="icon"
                variant="ghost"
                class="size-8"
                title="Redo"
                :disabled="!editor.can().redo()"
                @click="editor.chain().focus().redo().run()"
                ><Redo2
            /></Button>
        </div>
        <EditorContent :editor="editor" />
    </div>
</template>

<style scoped>
:deep(.tiptap p.is-editor-empty:first-child::before) {
    content: attr(data-placeholder);
    float: left;
    height: 0;
    pointer-events: none;
    color: var(--muted-foreground);
}
</style>
