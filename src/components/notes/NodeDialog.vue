<script setup lang="ts">
import { ref, watch } from 'vue';
import { Button } from '@/components/ui/button';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useNotes } from '@/composables/useNotes';

const props = defineProps<{
    open: boolean;
    /** Existing node being edited, or null when creating a child of `parentId`. */
    node: { id: string; title: string } | null;
    parentId: string | null;
}>();

const emit = defineEmits<{ 'update:open': [value: boolean] }>();

const { createNode, renameNode } = useNotes();

const title = ref('');
const processing = ref(false);

watch(
    () => props.open,
    (open) => {
        if (open) {
            title.value = props.node?.title ?? '';
        }
    },
);

async function submit(): Promise<void> {
    processing.value = true;

    const saved = props.node
        ? await renameNode(props.node.id, title.value)
        : props.parentId
          ? await createNode(props.parentId, title.value)
          : false;

    processing.value = false;

    if (saved) {
        emit('update:open', false);
    }
}
</script>

<template>
    <Dialog :open="open" @update:open="emit('update:open', $event)">
        <DialogContent class="sm:max-w-md">
            <form class="space-y-4" @submit.prevent="submit">
                <DialogHeader>
                    <DialogTitle>{{
                        node ? 'Edit node' : 'New node'
                    }}</DialogTitle>
                    <DialogDescription>
                        Nodes group topics and form the tree on the left.
                    </DialogDescription>
                </DialogHeader>

                <div class="grid gap-2">
                    <Label for="node-title">Title</Label>
                    <Input
                        id="node-title"
                        v-model="title"
                        maxlength="255"
                        required
                        autofocus
                    />
                </div>

                <DialogFooter>
                    <Button
                        type="button"
                        variant="outline"
                        @click="emit('update:open', false)"
                        >Cancel</Button
                    >
                    <Button type="submit" :disabled="processing">Save</Button>
                </DialogFooter>
            </form>
        </DialogContent>
    </Dialog>
</template>
