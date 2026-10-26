<script setup lang="ts">
import { ChevronRight, Folder } from '@lucide/vue';
import { computed } from 'vue';
import { cn } from '@/lib/utils';
import type { TreeNode } from '@/types/notes';

type Branch = TreeNode & { children: Branch[] };

const props = defineProps<{
    node: Branch;
    selectedId: string | null;
    expanded: Set<string>;
    cutId: string | null;
    depth?: number;
}>();

const emit = defineEmits<{
    select: [id: string];
    toggle: [id: string];
}>();

const isOpen = computed(() => props.expanded.has(props.node.id));
const hasChildren = computed(() => props.node.children.length > 0);
const depth = computed(() => props.depth ?? 0);
</script>

<template>
    <li role="treeitem" :aria-expanded="hasChildren ? isOpen : undefined">
        <div
            :class="
                cn(
                    'group flex cursor-pointer items-center gap-1 rounded-md py-1 pr-2 text-sm hover:bg-accent',
                    node.id === selectedId && 'bg-accent font-medium',
                    node.id === cutId && 'opacity-50',
                )
            "
            :style="{ paddingLeft: `${depth * 14 + 4}px` }"
            @click="emit('select', node.id)"
        >
            <button
                type="button"
                class="flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:text-foreground"
                :class="{ invisible: !hasChildren }"
                :aria-label="isOpen ? 'Collapse' : 'Expand'"
                @click.stop="emit('toggle', node.id)"
            >
                <ChevronRight
                    class="size-4 transition-transform"
                    :class="{ 'rotate-90': isOpen }"
                />
            </button>
            <Folder class="size-3.5 shrink-0 text-muted-foreground" />
            <span class="truncate">{{ node.title }}</span>
        </div>
        <ul v-if="hasChildren && isOpen" role="group">
            <NodeTreeItem
                v-for="child in node.children"
                :key="child.id"
                :node="child"
                :selected-id="selectedId"
                :expanded="expanded"
                :cut-id="cutId"
                :depth="depth + 1"
                @select="emit('select', $event)"
                @toggle="emit('toggle', $event)"
            />
        </ul>
    </li>
</template>
