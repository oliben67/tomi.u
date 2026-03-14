import * as vscode from 'vscode';
import * as path from 'path';
import { 
    LanguageClient, 
    LanguageClientOptions, 
    ServerOptions, 
    TransportKind 
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    console.log('Tomi Language Support extension is now active!');
    
    // Create output channel
    outputChannel = vscode.window.createOutputChannel('Tomi Language Server');
    context.subscriptions.push(outputChannel);

    // Start the language server
    startLanguageServer(context);

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('tomi.restartLanguageServer', () => {
            restartLanguageServer(context);
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('tomi.showOutputChannel', () => {
            outputChannel.show();
        })
    );

    // Register document formatters and providers
    registerProviders(context);
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function startLanguageServer(context: vscode.ExtensionContext) {
    // Path to the Tomi compiler binary
    const tomiPath = getTomiCompilerPath();
    
    if (!tomiPath) {
        vscode.window.showErrorMessage(
            'Tomi compiler not found. Please ensure tomi is in your PATH or set tomi.compilerPath.'
        );
        return;
    }

    // Server options
    const serverOptions: ServerOptions = {
        command: tomiPath,
        args: ['--language-server'],
        options: {
            env: {
                ...process.env,
                TOMI_LSP_MODE: '1'
            }
        }
    };

    // Language client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'tomi' },
            { scheme: 'untitled', language: 'tomi' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.tomi')
        },
        outputChannel: outputChannel,
        initializationOptions: getConfiguration()
    };

    // Create and start the language client
    client = new LanguageClient(
        'tomiLanguageServer',
        'Tomi Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client and language server
    client.start().then(() => {
        outputChannel.appendLine('Tomi Language Server started successfully');
    }).catch((error) => {
        outputChannel.appendLine(`Failed to start Tomi Language Server: ${error}`);
        vscode.window.showErrorMessage(`Tomi Language Server failed to start: ${error.message}`);
    });

    context.subscriptions.push(client);
}

function restartLanguageServer(context: vscode.ExtensionContext) {
    if (client) {
        client.stop().then(() => {
            startLanguageServer(context);
        });
    } else {
        startLanguageServer(context);
    }
}

function getTomiCompilerPath(): string | undefined {
    const config = vscode.workspace.getConfiguration('tomi');
    const customPath = config.get<string>('compilerPath');
    
    if (customPath) {
        return customPath;
    }
    
    // Try to find tomi in the workspace or PATH
    const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspaceRoot) {
        const workspaceTomi = path.join(workspaceRoot, 'target', 'debug', 'tomi');
        if (require('fs').existsSync(workspaceTomi)) {
            return workspaceTomi;
        }
    }
    
    return 'tomi'; // Assume it's in PATH
}

function getConfiguration() {
    const config = vscode.workspace.getConfiguration('tomi');
    return {
        analysis: {
            typeCheckingMode: config.get('analysis.typeCheckingMode', 'basic'),
            autoImportCompletions: config.get('analysis.autoImportCompletions', true),
            diagnosticMode: config.get('analysis.diagnosticMode', 'openFilesOnly'),
            logLevel: config.get('analysis.logLevel', 'Information')
        },
        completion: {
            includeSnippets: config.get('completion.includeSnippets', true)
        },
        hover: {
            includeTypes: config.get('hover.includeTypes', true)
        }
    };
}

function registerProviders(context: vscode.ExtensionContext) {
    // Register code action provider for quick fixes
    const codeActionProvider = vscode.languages.registerCodeActionsProvider('tomi', {
        provideCodeActions(document: vscode.TextDocument, range: vscode.Range | vscode.Selection, context: vscode.CodeActionContext) {
            const actions: vscode.CodeAction[] = [];
            
            for (const diagnostic of context.diagnostics) {
                if (diagnostic.message.includes('type mismatch')) {
                    const action = new vscode.CodeAction('Add type annotation', vscode.CodeActionKind.QuickFix);
                    action.edit = new vscode.WorkspaceEdit();
                    action.diagnostics = [diagnostic];
                    actions.push(action);
                }
                
                if (diagnostic.message.includes('undefined')) {
                    const action = new vscode.CodeAction('Import module', vscode.CodeActionKind.QuickFix);
                    action.edit = new vscode.WorkspaceEdit();
                    action.diagnostics = [diagnostic];
                    actions.push(action);
                }
            }
            
            return actions;
        }
    });
    
    context.subscriptions.push(codeActionProvider);

    // Register folding range provider
    const foldingProvider = vscode.languages.registerFoldingRangeProvider('tomi', {
        provideFoldingRanges(document: vscode.TextDocument) {
            const ranges: vscode.FoldingRange[] = [];
            let indentStack: { line: number, indent: number }[] = [];
            
            for (let i = 0; i < document.lineCount; i++) {
                const line = document.lineAt(i);
                const indent = line.firstNonWhitespaceCharacterIndex;
                
                if (line.text.trim().endsWith(':')) {
                    // Start of a new block
                    while (indentStack.length > 0 && indentStack[indentStack.length - 1].indent >= indent) {
                        const start = indentStack.pop()!;
                        ranges.push(new vscode.FoldingRange(start.line, i - 1));
                    }
                    indentStack.push({ line: i, indent });
                }
            }
            
            // Close remaining ranges
            while (indentStack.length > 0) {
                const start = indentStack.pop()!;
                ranges.push(new vscode.FoldingRange(start.line, document.lineCount - 1));
            }
            
            return ranges;
        }
    });
    
    context.subscriptions.push(foldingProvider);

    // Register document symbol provider
    const symbolProvider = vscode.languages.registerDocumentSymbolProvider('tomi', {
        provideDocumentSymbols(document: vscode.TextDocument) {
            const symbols: vscode.DocumentSymbol[] = [];
            
            for (let i = 0; i < document.lineCount; i++) {
                const line = document.lineAt(i);
                const text = line.text.trim();
                
                // Function definitions
                const funcMatch = text.match(/^(?:::[\w\[\]]*\s+)?def\s+([a-zA-Z_]\w*)/);
                if (funcMatch) {
                    const name = funcMatch[1];
                    const range = line.range;
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        '',
                        vscode.SymbolKind.Function,
                        range,
                        range
                    );
                    symbols.push(symbol);
                }
                
                // Class definitions
                const classMatch = text.match(/^class\s+([a-zA-Z_]\w*)/);
                if (classMatch) {
                    const name = classMatch[1];
                    const range = line.range;
                    const symbol = new vscode.DocumentSymbol(
                        name,
                        '',
                        vscode.SymbolKind.Class,
                        range,
                        range
                    );
                    symbols.push(symbol);
                }
            }
            
            return symbols;
        }
    });
    
    context.subscriptions.push(symbolProvider);
}