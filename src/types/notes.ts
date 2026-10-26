export type TreeNode = {
    id: string;
    parent_id: string | null;
    title: string;
    can_edit: boolean;
};

export type TopicSummary = {
    id: string;
    title: string;
    is_link: boolean;
    is_favorite: boolean;
};

export type Attachment = {
    id: string;
    name: string;
    size: number;
};

export type TopicDetail = {
    id: string;
    title: string;
    content: string;
    is_link: boolean;
    can_edit: boolean;
    updated_at: string;
    tags: string[];
    attachments: Attachment[];
};

export type TopicInput = {
    title: string;
    content: string;
    tags: string[];
};

export type SearchMode = 'phrase' | 'all';

export type SearchQuery = {
    q: string;
    tag: string;
    fav: boolean;
    mode: SearchMode;
    title_only: boolean;
};

export type SearchResult = {
    topic_id: string;
    node_id: string;
    title: string;
    path: string;
};

export type SearchState = SearchQuery & { results: SearchResult[] };

export type PasteMode = 'move' | 'copy' | 'link';

export type Clipboard =
    | { kind: 'node'; id: string; title: string }
    | {
          kind: 'topic';
          id: string;
          title: string;
          mode: PasteMode;
          fromNodeId: string;
      };
