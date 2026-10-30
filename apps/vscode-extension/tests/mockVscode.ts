/**
 * Comprehensive mock for the VS Code extension API for unit and integration testing.
 */

export interface MockQuickPick<T> {
  title?: string;
  placeholder?: string;
  items: T[];
  busy: boolean;
  matchOnDescription: boolean;
  matchOnDetail: boolean;
  selectedItems: T[];
  show: () => void;
  hide: () => void;
  dispose: () => void;
  onDidChangeValue: (cb: (val: string) => void) => void;
  onDidAccept: (cb: () => void) => void;
  onDidHide: (cb: () => void) => void;
  _triggerChangeValue?: (val: string) => void;
  _triggerAccept?: () => void;
}

export class MockEventEmitter<T> {
  private readonly listeners: ((val: T) => void)[] = [];

  public event = (listener: (val: T) => void): { dispose: () => void } => {
    this.listeners.push(listener);
    return {
      dispose: () => {
        const idx = this.listeners.indexOf(listener);
        if (idx >= 0) {
          this.listeners.splice(idx, 1);
        }
      },
    };
  };

  public fire(val: T): void {
    for (const listener of this.listeners) {
      listener(val);
    }
  }

  public dispose(): void {
    this.listeners.length = 0;
  }
}

export class MockTreeItem {
  public label?: string;
  public description?: string;
  public tooltip?: string;
  public iconPath?: unknown;
  public contextValue?: string;
  public id?: string;
  public command?: unknown;

  constructor(
    label?: string,
    public collapsibleState: number = 0
  ) {
    this.label = label;
  }
}

export const mockVscode = {
  TreeItem: MockTreeItem,
  TreeItemCollapsibleState: {
    None: 0,
    Collapsed: 1,
    Expanded: 2,
  },
  ThemeIcon: class {
    constructor(
      public id: string,
      public color?: unknown
    ) {}
  },
  ThemeColor: class {
    constructor(public id: string) {}
  },
  EventEmitter: MockEventEmitter,
  Uri: {
    file: (fsPath: string) => ({ fsPath, scheme: 'file', toString: () => fsPath }),
    parse: (url: string) => ({ fsPath: url, scheme: 'http', path: url, toString: () => url }),
  },
  Range: class {
    constructor(
      public startLine: number,
      public startCol: number,
      public endLine: number,
      public endCol: number
    ) {}
  },
  StatusBarAlignment: {
    Left: 1,
    Right: 2,
  },
  ProgressLocation: {
    Notification: 15,
  },
  workspace: {
    workspaceFolders: [] as { uri: { fsPath: string }; name: string }[],
    isTrusted: true,
    getConfiguration: (_section?: string) => ({
      get: <T>(_key: string, defaultValue?: T): T => defaultValue as T,
    }),
    onDidChangeWorkspaceFolders: new MockEventEmitter<unknown>().event,
    onDidChangeConfiguration: new MockEventEmitter<unknown>().event,
    asRelativePath: (uri: { fsPath: string }) => uri.fsPath,
    openTextDocument: async (doc: unknown) => doc,
    getWorkspaceFolder: (uri: { fsPath: string }) => {
      const folders = mockVscode.workspace.workspaceFolders || [];
      return folders.find((f) => uri.fsPath.startsWith(f.uri.fsPath));
    },
  },
  window: {
    activeTextEditor: undefined as unknown,
    showInformationMessage: async (_msg: string, ..._items: string[]) => undefined,
    showWarningMessage: async (_msg: string, ..._items: string[]) => undefined,
    showErrorMessage: async (_msg: string, ..._items: string[]) => undefined,
    showInputBox: async (_options?: unknown) => undefined,
    showQuickPick: async (_items: unknown, _options?: unknown) => undefined,
    showOpenDialog: async (_options?: unknown) => undefined,
    withProgress: async (_opts: unknown, task: () => Promise<unknown>) => task(),
    createStatusBarItem: () => ({
      text: '',
      tooltip: '',
      command: '',
      backgroundColor: undefined,
      show: () => {},
      hide: () => {},
      dispose: () => {},
    }),
    createOutputChannel: (name: string) => ({
      name,
      appendLine: (_line: string) => {},
      clear: () => {},
      show: () => {},
      dispose: () => {},
    }),
    registerTreeDataProvider: (_viewId: string, _provider: unknown) => ({
      dispose: () => {},
    }),
    createQuickPick: <T>() => {
      const listeners = {
        change: [] as ((val: string) => void)[],
        accept: [] as (() => void)[],
        hide: [] as (() => void)[],
      };

      const qp: MockQuickPick<T> = {
        title: '',
        placeholder: '',
        items: [],
        busy: false,
        matchOnDescription: true,
        matchOnDetail: true,
        selectedItems: [],
        show: () => {},
        hide: () => {
          for (const cb of listeners.hide) cb();
        },
        dispose: () => {},
        onDidChangeValue: (cb) => {
          listeners.change.push(cb);
        },
        onDidAccept: (cb) => {
          listeners.accept.push(cb);
        },
        onDidHide: (cb) => {
          listeners.hide.push(cb);
        },
        _triggerChangeValue: (val) => {
          for (const cb of listeners.change) cb(val);
        },
        _triggerAccept: () => {
          for (const cb of listeners.accept) cb();
        },
      };
      return qp;
    },
    showTextDocument: async (_doc: unknown) => undefined,
  },
  commands: {
    registered: new Map<string, (...args: unknown[]) => unknown>(),
    registerCommand: (command: string, callback: (...args: unknown[]) => unknown) => {
      mockVscode.commands.registered.set(command, callback);
      return {
        dispose: () => {
          mockVscode.commands.registered.delete(command);
        },
      };
    },
    executeCommand: async (command: string, ...args: unknown[]) => {
      const handler = mockVscode.commands.registered.get(command);
      if (handler) {
        return handler(...args);
      }
      return undefined;
    },
  },
  env: {
    openExternal: async (_uri: unknown) => true,
  },
};
