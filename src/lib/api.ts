import { convertFileSrc, invoke } from '@tauri-apps/api/core';
import type {
    Attachment,
    PasteMode,
    SearchQuery,
    SearchResult,
    TopicDetail,
    TopicInput,
    TopicSummary,
    TreeNode,
} from '@/types/notes';

/** Typed wrappers around the Rust commands; every failure rejects with a message string. */
export const api = {
    listNodes: () => invoke<TreeNode[]>('list_nodes'),
    createNode: (parentId: string, title: string) =>
        invoke<string>('create_node', { parentId, title }),
    renameNode: (id: string, title: string) =>
        invoke<void>('rename_node', { id, title }),
    moveNode: (id: string, parentId: string) =>
        invoke<void>('move_node', { id, parentId }),
    deleteNode: (id: string) => invoke<string>('delete_node', { id }),

    listTopics: (nodeId: string) =>
        invoke<TopicSummary[]>('list_topics', { nodeId }),
    getTopic: (id: string, nodeId: string | null) =>
        invoke<TopicDetail | null>('get_topic', { id, nodeId }),
    createTopic: (nodeId: string, input: TopicInput) =>
        invoke<string>('create_topic', { nodeId, input }),
    updateTopic: (id: string, input: TopicInput) =>
        invoke<void>('update_topic', { id, input }),
    deleteTopic: (id: string, nodeId: string) =>
        invoke<void>('delete_topic', { id, nodeId }),
    pasteTopic: (
        id: string,
        mode: PasteMode,
        nodeId: string,
        fromNodeId: string | null,
    ) => invoke<string>('paste_topic', { id, mode, nodeId, fromNodeId }),
    toggleFavorite: (id: string) => invoke<void>('toggle_favorite', { id }),
    knownTags: () => invoke<string[]>('known_tags'),
    search: (query: SearchQuery) =>
        invoke<SearchResult[]>('search_topics', { query }),

    addAttachment: (topicId: string, path: string) =>
        invoke<Attachment>('add_attachment', { topicId, path }),
    /** Stores an image and resolves with its media id. */
    async addImage(topicId: string, file: File): Promise<string> {
        return invoke<string>('add_image', new Uint8Array(await file.arrayBuffer()), {
            headers: {
                'x-topic-id': topicId,
                'x-file-name': encodeURIComponent(file.name),
            },
        });
    },
    deleteMedia: (id: string) => invoke<void>('delete_media', { id }),
    exportMedia: (id: string, path: string) =>
        invoke<void>('export_media', { id, path }),
};

export function errorMessage(error: unknown): string {
    return typeof error === 'string'
        ? error
        : error instanceof Error
          ? error.message
          : 'Something went wrong.';
}

/* Topic content is stored with relative `/media/<id>` urls; the webview reaches the files
   through the custom `media` protocol, so the urls are swapped on the way in and out. */

const mediaBase = convertFileSrc('', 'media');

export function mediaUrl(id: string): string {
    return mediaBase + id;
}

const storedSrc = /(src=")\/media\/([\w-]+)"/g;
const displaySrc = new RegExp(
    `(src=")${mediaBase.replace(/[.*+?^${}()|[\]\\/]/g, '\\$&')}([\\w-]+)"`,
    'g',
);

export function toDisplayHtml(html: string): string {
    return html.replace(storedSrc, (_, attr, id) => `${attr}${mediaUrl(id)}"`);
}

export function toStoredHtml(html: string): string {
    return html.replace(displaySrc, (_, attr, id) => `${attr}/media/${id}"`);
}
