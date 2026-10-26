import { ref } from 'vue';
import { toast } from 'vue-sonner';
import { api, errorMessage } from '@/lib/api';
import type {
    PasteMode,
    SearchState,
    TopicDetail,
    TopicInput,
    TopicSummary,
    TreeNode,
} from '@/types/notes';

/*
 * Application state of the notes screen. It lives at module level so the selection
 * survives navigating to the settings and back.
 */

const tree = ref<TreeNode[]>([]);
const nodeId = ref<string | null>(null);
const topics = ref<TopicSummary[]>([]);
const topicId = ref<string | null>(null);
const topic = ref<TopicDetail | null>(null);
const search = ref<SearchState | null>(null);
const knownTags = ref<string[]>([]);

/** Runs a backend call, reporting a failure as a toast instead of throwing. */
async function attempt<T>(call: () => Promise<T>): Promise<T | undefined> {
    try {
        return await call();
    } catch (error) {
        toast.error(errorMessage(error));

        return undefined;
    }
}

async function refreshTree(): Promise<void> {
    tree.value = (await attempt(api.listNodes)) ?? tree.value;
}

async function refreshTopics(): Promise<void> {
    topics.value = nodeId.value
        ? ((await attempt(() => api.listTopics(nodeId.value!))) ?? [])
        : [];
}

async function refreshTopic(): Promise<void> {
    topic.value = topicId.value
        ? ((await attempt(() => api.getTopic(topicId.value!, nodeId.value))) ??
          null)
        : null;

    if (!topic.value) {
        topicId.value = null;
    }
}

async function refreshTags(): Promise<void> {
    knownTags.value = (await attempt(api.knownTags)) ?? [];
}

async function init(): Promise<void> {
    await refreshTree();

    if (!nodeId.value || !tree.value.some((n) => n.id === nodeId.value)) {
        nodeId.value = tree.value.find((n) => n.parent_id === null)?.id ?? null;
    }

    await Promise.all([refreshTopics(), refreshTopic(), refreshTags()]);
}

async function selectNode(id: string): Promise<void> {
    nodeId.value = id;
    topicId.value = null;
    search.value = null;
    await Promise.all([refreshTopics(), refreshTopic()]);
}

/** Opens a topic, switching to `inNodeId` when it is given (search results). */
async function selectTopic(id: string, inNodeId = nodeId.value): Promise<void> {
    const changedNode = inNodeId !== nodeId.value;

    nodeId.value = inNodeId;
    topicId.value = id;
    await Promise.all([changedNode ? refreshTopics() : null, refreshTopic()]);
}

async function runSearch(query: Omit<SearchState, 'results'>): Promise<void> {
    const results = await attempt(() => api.search(query));

    if (results) {
        search.value = { ...query, results };
    }
}

function clearSearch(): void {
    search.value = null;
}

/* ---------- nodes ---------- */

async function createNode(parentId: string, title: string): Promise<boolean> {
    const id = await attempt(() => api.createNode(parentId, title));

    if (!id) {
        return false;
    }

    await refreshTree();
    await selectNode(id);

    return true;
}

async function renameNode(id: string, title: string): Promise<boolean> {
    const done = await attempt(async () => {
        await api.renameNode(id, title);

        return true;
    });

    await refreshTree();

    return done === true;
}

async function moveNode(id: string, parentId: string): Promise<boolean> {
    const done = await attempt(async () => {
        await api.moveNode(id, parentId);

        return true;
    });

    if (done) {
        await refreshTree();
        await selectNode(id);
    }

    return done === true;
}

async function deleteNode(id: string): Promise<void> {
    const parentId = await attempt(() => api.deleteNode(id));

    if (parentId) {
        await refreshTree();
        await selectNode(parentId);
        await refreshTags();
    }
}

/* ---------- topics ---------- */

/** Creates (when `id` is null) or updates a topic; resolves with its id. */
async function saveTopic(
    id: string | null,
    inNodeId: string,
    input: TopicInput,
): Promise<string | undefined> {
    const saved = await attempt(async () => {
        if (id) {
            await api.updateTopic(id, input);

            return id;
        }

        return api.createTopic(inNodeId, input);
    });

    if (saved) {
        topicId.value = saved;
        await Promise.all([refreshTopics(), refreshTopic(), refreshTags()]);
    }

    return saved;
}

async function deleteTopic(id: string, inNodeId: string): Promise<boolean> {
    const done = await attempt(async () => {
        await api.deleteTopic(id, inNodeId);

        return true;
    });

    if (done) {
        topicId.value = null;
        await Promise.all([refreshTopics(), refreshTopic(), refreshTags()]);
    }

    return done === true;
}

async function pasteTopic(
    id: string,
    mode: PasteMode,
    inNodeId: string,
    fromNodeId: string | null,
): Promise<boolean> {
    const pasted = await attempt(() =>
        api.pasteTopic(id, mode, inNodeId, fromNodeId),
    );

    if (pasted) {
        await selectNode(inNodeId);
        await selectTopic(pasted);
    }

    return !!pasted;
}

async function toggleFavorite(id: string): Promise<void> {
    await attempt(() => api.toggleFavorite(id));
    await refreshTopics();
}

export function useNotes() {
    return {
        tree,
        nodeId,
        topics,
        topic,
        search,
        knownTags,
        init,
        selectNode,
        selectTopic,
        refreshTopic,
        runSearch,
        clearSearch,
        createNode,
        renameNode,
        moveNode,
        deleteNode,
        saveTopic,
        deleteTopic,
        pasteTopic,
        toggleFavorite,
    };
}
