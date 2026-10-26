<script setup lang="ts">
import {
    ClipboardPaste,
    Copy,
    FilePlus2,
    FolderPlus,
    Link2,
    Pencil,
    Scissors,
    Search,
    Star,
    Trash2,
    X,
} from '@lucide/vue';
import { computed, onMounted, ref, watch } from 'vue';
import { toast } from 'vue-sonner';
import ConfirmDialog from '@/components/notes/ConfirmDialog.vue';
import NodeDialog from '@/components/notes/NodeDialog.vue';
import NodeTreeItem from '@/components/notes/NodeTreeItem.vue';
import TopicEditor from '@/components/notes/TopicEditor.vue';
import TopicViewer from '@/components/notes/TopicViewer.vue';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { useNotes } from '@/composables/useNotes';
import { cn } from '@/lib/utils';
import type { Clipboard, SearchMode, TreeNode } from '@/types/notes';

type Branch = TreeNode & { children: Branch[] };

const {
    tree,
    nodeId,
    topics,
    topic,
    search,
    knownTags,
    init,
    selectNode,
    selectTopic,
    runSearch,
    clearSearch,
    moveNode,
    deleteNode,
    deleteTopic,
    pasteTopic,
    toggleFavorite,
} = useNotes();

onMounted(init);

/* ---------- tree ---------- */

const byId = computed(() => new Map(tree.value.map((n) => [n.id, n])));

const branches = computed<Branch[]>(() => {
    const nodes = new Map<string, Branch>(
        tree.value.map((n) => [n.id, { ...n, children: [] }]),
    );
    const roots: Branch[] = [];

    for (const node of nodes.values()) {
        const parent = node.parent_id ? nodes.get(node.parent_id) : null;
        (parent ? parent.children : roots).push(node);
    }

    return roots;
});

const selectedNode = computed(() =>
    nodeId.value ? (byId.value.get(nodeId.value) ?? null) : null,
);

const expanded = ref(new Set<string>());

function expandPathTo(id: string | null): void {
    const next = new Set(expanded.value);
    let node = id ? byId.value.get(id) : undefined;

    while (node) {
        next.add(node.id);
        node = node.parent_id ? byId.value.get(node.parent_id) : undefined;
    }

    expanded.value = next;
}

function toggle(id: string): void {
    const next = new Set(expanded.value);
    if (!next.delete(id)) {
        next.add(id);
    }
    expanded.value = next;
}

watch([nodeId, tree], () => expandPathTo(nodeId.value), { immediate: true });

/* ---------- navigation ---------- */

function onSelectNode(id: string): void {
    editing.value = null;
    void selectNode(id);
}

function onSelectTopic(id: string, inNodeId: string | null = nodeId.value): void {
    editing.value = null;
    void selectTopic(id, inNodeId);
}

/* ---------- search ---------- */

const query = ref('');
const titleOnly = ref(false);
const mode = ref<SearchMode>('phrase');

watch(search, (state) => {
    query.value = state?.q ?? '';
    titleOnly.value = state?.title_only ?? false;
    mode.value = state?.mode ?? 'phrase';
});

function find(overrides: { q?: string; tag?: string; fav?: boolean }): void {
    editing.value = null;
    void runSearch({
        q: '',
        tag: '',
        fav: false,
        mode: mode.value,
        title_only: titleOnly.value,
        ...overrides,
    });
}

function runTextSearch(): void {
    if (!query.value.trim()) {
        return clearSearch();
    }

    find({ q: query.value.trim() });
}

const searchTag = (tag: string) => find({ tag });
const searchFavorites = () => find({ fav: true });

/* ---------- clipboard ---------- */

const clipboard = ref<Clipboard | null>(null);

const cutNodeId = computed(() =>
    clipboard.value?.kind === 'node' ? clipboard.value.id : null,
);

function cutNode(): void {
    if (selectedNode.value) {
        clipboard.value = {
            kind: 'node',
            id: selectedNode.value.id,
            title: selectedNode.value.title,
        };
    }
}

function stash(pasteMode: 'move' | 'copy' | 'link'): void {
    if (topic.value && nodeId.value) {
        clipboard.value = {
            kind: 'topic',
            id: topic.value.id,
            title: topic.value.title,
            mode: pasteMode,
            fromNodeId: nodeId.value,
        };
    }
}

async function pasteNode(): Promise<void> {
    if (clipboard.value?.kind !== 'node' || !selectedNode.value) {
        return;
    }

    if (await moveNode(clipboard.value.id, selectedNode.value.id)) {
        clipboard.value = null;
    }
}

async function pasteIntoNode(): Promise<void> {
    const item = clipboard.value;

    if (item?.kind !== 'topic' || !nodeId.value) {
        return;
    }

    const pasted = await pasteTopic(
        item.id,
        item.mode,
        nodeId.value,
        item.fromNodeId,
    );

    if (pasted && item.mode === 'move') {
        clipboard.value = null;
    }
}

/* ---------- node dialog / deletion ---------- */

const nodeDialog = ref<{ open: boolean; editing: boolean }>({
    open: false,
    editing: false,
});
const confirm = ref<{ open: boolean; kind: 'node' | 'topic' }>({
    open: false,
    kind: 'node',
});

function openNodeDialog(isEdit: boolean): void {
    nodeDialog.value = { open: true, editing: isEdit };
}

async function deleteSelected(): Promise<void> {
    if (confirm.value.kind === 'node' && selectedNode.value) {
        await deleteNode(selectedNode.value.id);
    } else if (confirm.value.kind === 'topic' && topic.value && nodeId.value) {
        if (await deleteTopic(topic.value.id, nodeId.value)) {
            toast.success('Topic removed.');
        }
    }
}

function askDelete(kind: 'node' | 'topic'): void {
    confirm.value = { open: true, kind };
}

/* ---------- topics ---------- */

const editing = ref<'new' | 'edit' | null>(null);

const canAddTopic = computed(() => !!selectedNode.value);
const pasteTopicLabel = computed(() => {
    const item = clipboard.value;

    if (item?.kind !== 'topic') {
        return '';
    }

    return {
        move: 'Paste (move)',
        copy: 'Paste as copy',
        link: 'Paste as link',
    }[item.mode];
});

const topicsSelectedId = computed(() => topic.value?.id ?? null);

</script>

<template>
    <div class="flex h-[calc(100svh-4rem)] flex-col gap-3 p-4">
        <!-- search -->
        <form
            class="flex flex-wrap items-center gap-2"
            @submit.prevent="runTextSearch"
        >
            <div class="relative w-full max-w-sm">
                <Search
                    class="absolute top-2.5 left-2.5 size-4 text-muted-foreground"
                />
                <Input
                    v-model="query"
                    class="pl-8"
                    placeholder="Search topics"
                />
            </div>
            <Select v-model="mode">
                <SelectTrigger class="w-44"><SelectValue /></SelectTrigger>
                <SelectContent>
                    <SelectItem value="phrase">Exact phrase</SelectItem>
                    <SelectItem value="all">All words</SelectItem>
                </SelectContent>
            </Select>
            <label class="flex items-center gap-2 text-sm">
                <Checkbox
                    :model-value="titleOnly"
                    @update:model-value="titleOnly = $event === true"
                />
                Titles only
            </label>
            <Button type="submit" size="sm">Search</Button>
            <Button
                type="button"
                size="sm"
                variant="outline"
                title="Favorites"
                @click="searchFavorites"
            >
                <Star /> Favorites
            </Button>
            <Button
                v-if="search"
                type="button"
                size="sm"
                variant="ghost"
                @click="clearSearch"
            >
                <X /> Clear
            </Button>
        </form>

        <div
            class="grid min-h-0 flex-1 gap-3 lg:grid-cols-[18rem_20rem_minmax(0,1fr)]"
        >
            <!-- tree -->
            <section class="flex min-h-0 flex-col rounded-lg border">
                <div class="flex items-center gap-1 border-b p-1.5">
                    <Button
                        size="icon"
                        variant="ghost"
                        class="size-8"
                        title="New node"
                        :disabled="!selectedNode"
                        @click="openNodeDialog(false)"
                        ><FolderPlus
                    /></Button>
                    <Button
                        size="icon"
                        variant="ghost"
                        class="size-8"
                        title="Edit node"
                        :disabled="!selectedNode?.can_edit"
                        @click="openNodeDialog(true)"
                        ><Pencil
                    /></Button>
                    <Button
                        size="icon"
                        variant="ghost"
                        class="size-8"
                        title="Cut node"
                        :disabled="!selectedNode?.can_edit"
                        @click="cutNode"
                        ><Scissors
                    /></Button>
                    <Button
                        size="icon"
                        variant="ghost"
                        class="size-8"
                        title="Paste node here"
                        :disabled="clipboard?.kind !== 'node' || !selectedNode"
                        @click="pasteNode"
                        ><ClipboardPaste
                    /></Button>
                    <Button
                        size="icon"
                        variant="ghost"
                        class="size-8 text-destructive"
                        title="Delete node"
                        :disabled="!selectedNode?.can_edit"
                        @click="askDelete('node')"
                        ><Trash2
                    /></Button>
                </div>
                <ScrollArea class="min-h-0 flex-1">
                    <ul role="tree" class="p-1.5">
                        <NodeTreeItem
                            v-for="branch in branches"
                            :key="branch.id"
                            :node="branch"
                            :selected-id="nodeId"
                            :expanded="expanded"
                            :cut-id="cutNodeId"
                            @select="onSelectNode"
                            @toggle="toggle"
                        />
                    </ul>
                </ScrollArea>
            </section>

            <!-- topics / search results -->
            <section class="flex min-h-0 flex-col rounded-lg border">
                <template v-if="search">
                    <div
                        class="flex items-center gap-2 border-b px-3 py-2 text-sm"
                    >
                        <Search class="size-4 text-muted-foreground" />
                        <span class="truncate">
                            <template v-if="search.fav">Favorites</template>
                            <template v-else-if="search.tag"
                                >Tag: {{ search.tag }}</template
                            >
                            <template v-else>“{{ search.q }}”</template>
                        </span>
                        <Badge variant="secondary" class="ml-auto">{{
                            search.results.length
                        }}</Badge>
                    </div>
                    <ScrollArea class="min-h-0 flex-1">
                        <p
                            v-if="!search.results.length"
                            class="p-4 text-sm text-muted-foreground"
                        >
                            Nothing found.
                        </p>
                        <ul class="p-1.5">
                            <li
                                v-for="r in search.results"
                                :key="r.topic_id + r.node_id"
                            >
                                <button
                                    type="button"
                                    :class="
                                        cn(
                                            'w-full rounded-md px-2 py-1.5 text-left hover:bg-accent',
                                            r.topic_id === topicsSelectedId &&
                                                'bg-accent',
                                        )
                                    "
                                    @click="onSelectTopic(r.topic_id, r.node_id)"
                                >
                                    <div class="truncate text-sm font-medium">
                                        {{ r.title }}
                                    </div>
                                    <div
                                        class="truncate text-xs text-muted-foreground"
                                    >
                                        {{ r.path }}
                                    </div>
                                </button>
                            </li>
                        </ul>
                    </ScrollArea>
                </template>

                <template v-else>
                    <div
                        class="flex flex-wrap items-center gap-1 border-b p-1.5"
                    >
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8"
                            title="New topic"
                            :disabled="!canAddTopic"
                            @click="editing = 'new'"
                            ><FilePlus2
                        /></Button>
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8"
                            title="Edit topic"
                            :disabled="!topic?.can_edit"
                            @click="editing = 'edit'"
                            ><Pencil
                        /></Button>
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8"
                            title="Cut topic"
                            :disabled="!topic?.can_edit"
                            @click="stash('move')"
                            ><Scissors
                        /></Button>
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8"
                            title="Copy topic (paste as a copy)"
                            :disabled="!topic || topic.is_link"
                            @click="stash('copy')"
                            ><Copy
                        /></Button>
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8"
                            title="Copy as link (paste as a link)"
                            :disabled="!topic || topic.is_link"
                            @click="stash('link')"
                            ><Link2
                        /></Button>
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8"
                            :title="pasteTopicLabel || 'Paste'"
                            :disabled="
                                clipboard?.kind !== 'topic' || !selectedNode
                            "
                            @click="pasteIntoNode"
                            ><ClipboardPaste
                        /></Button>
                        <Button
                            size="icon"
                            variant="ghost"
                            class="size-8 text-destructive"
                            title="Delete topic"
                            :disabled="
                                !topic || (!topic.can_edit && !topic.is_link)
                            "
                            @click="askDelete('topic')"
                            ><Trash2
                        /></Button>
                    </div>
                    <ScrollArea class="min-h-0 flex-1">
                        <p
                            v-if="!topics.length"
                            class="p-4 text-sm text-muted-foreground"
                        >
                            No topics in this node.
                        </p>
                        <ul class="p-1.5">
                            <li
                                v-for="t in topics"
                                :key="t.id"
                                class="group flex items-center"
                            >
                                <button
                                    type="button"
                                    :class="
                                        cn(
                                            'flex min-w-0 flex-1 items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent',
                                            t.id === topicsSelectedId &&
                                                'bg-accent font-medium',
                                        )
                                    "
                                    @click="onSelectTopic(t.id)"
                                >
                                    <Link2
                                        v-if="t.is_link"
                                        class="size-3.5 shrink-0 text-muted-foreground"
                                    />
                                    <span class="truncate">{{ t.title }}</span>
                                </button>
                                <button
                                    type="button"
                                    class="p-1.5 text-muted-foreground hover:text-foreground"
                                    :aria-label="
                                        t.is_favorite
                                            ? 'Remove from favorites'
                                            : 'Add to favorites'
                                    "
                                    @click="toggleFavorite(t.id)"
                                >
                                    <Star
                                        class="size-4"
                                        :class="
                                            t.is_favorite &&
                                            'fill-amber-400 text-amber-400'
                                        "
                                    />
                                </button>
                            </li>
                        </ul>
                    </ScrollArea>
                </template>
            </section>

            <!-- viewer / editor -->
            <section class="min-h-0 overflow-y-auto rounded-lg border p-4">
                <TopicEditor
                    v-if="editing === 'new' && selectedNode"
                    :key="`new-${selectedNode.id}`"
                    :topic="null"
                    :node-id="selectedNode.id"
                    :known-tags="knownTags"
                    @close="editing = null"
                />
                <TopicEditor
                    v-else-if="editing === 'edit' && topic && selectedNode"
                    :key="`edit-${topic.id}`"
                    :topic="topic"
                    :node-id="selectedNode.id"
                    :known-tags="knownTags"
                    @close="editing = null"
                />
                <TopicViewer
                    v-else-if="topic"
                    :topic="topic"
                    @tag="searchTag"
                />
                <p v-else class="text-sm text-muted-foreground">
                    Select a topic to read it, or create a new one in the
                    selected node.
                </p>
            </section>
        </div>
    </div>

    <NodeDialog
        v-model:open="nodeDialog.open"
        :node="nodeDialog.editing ? selectedNode : null"
        :parent-id="selectedNode?.id ?? null"
    />

    <ConfirmDialog
        v-model:open="confirm.open"
        :title="
            confirm.kind === 'node'
                ? 'Delete node?'
                : topic?.is_link
                  ? 'Remove link?'
                  : 'Delete topic?'
        "
        :description="
            confirm.kind === 'node'
                ? 'The node, everything below it and the topics stored in it will be deleted. Linked topics are only unlinked.'
                : topic?.is_link
                  ? 'The topic stays where it originally lives; only this link is removed.'
                  : 'The topic and its files will be deleted permanently.'
        "
        @confirm="deleteSelected"
    />
</template>
