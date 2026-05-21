import { createContext } from 'react';
import type { Column } from '../../pipeline-types';
import type { RepoItem } from '../../repo-types';

export type FieldContextValue = {
    upstreamSchema: Column[];
    nodeSchema: Column[];
    repoItems: RepoItem[];
};

export const FieldContext = createContext<FieldContextValue>({
    upstreamSchema: [],
    nodeSchema: [],
    repoItems: [],
});
