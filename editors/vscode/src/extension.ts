import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let statusBarItem: vscode.StatusBarItem;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel("Latch Language");
    outputChannel.appendLine("Latch extension activated.");

    // Status bar indicator
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = "$(gear~spin) Latch LSP";
    statusBarItem.command = 'latch.restartLsp';
    statusBarItem.tooltip = "Latch Language Server Protocol (Click to restart)";
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);

    // Start LSP Client if enabled
    startLspClient(context);

    // Register Commands
    context.subscriptions.push(
        vscode.commands.registerCommand('latch.run', () => runScript('run')),
        vscode.commands.registerCommand('latch.vm', () => runScript('vm')),
        vscode.commands.registerCommand('latch.check', () => runScript('check')),
        vscode.commands.registerCommand('latch.restartLsp', async () => {
            outputChannel.appendLine("Restarting Latch LSP Server...");
            if (client) {
                await client.stop();
                client = undefined;
            }
            await startLspClient(context);
            vscode.window.showInformationMessage("Latch LSP Server restarted.");
        })
    );
}

async function startLspClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('latch');
    const enabled = config.get<boolean>('lsp.enable', true);

    if (!enabled) {
        statusBarItem.text = "$(circle-slash) Latch LSP (Disabled)";
        return;
    }

    const executablePath = config.get<string>('executablePath', 'latch');

    const serverOptions: ServerOptions = {
        command: executablePath,
        args: ['lsp'],
        options: {
            shell: true
        }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'latch' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lt')
        },
        outputChannel: outputChannel
    };

    try {
        client = new LanguageClient(
            'latchLsp',
            'Latch Language Server',
            serverOptions,
            clientOptions
        );

        await client.start();
        statusBarItem.text = "$(check) Latch LSP";
        outputChannel.appendLine("Latch LSP server started successfully.");
    } catch (err: any) {
        statusBarItem.text = "$(warning) Latch LSP Error";
        outputChannel.appendLine(`Failed to start Latch LSP server: ${err?.message || err}`);
        vscode.window.showWarningMessage(
            `Could not start Latch Language Server ('${executablePath} lsp'). Ensure 'latch' CLI is installed in PATH.`
        );
    }
}

function runScript(mode: 'run' | 'vm' | 'check') {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        vscode.window.showErrorMessage("No active Latch file to execute.");
        return;
    }

    const document = editor.document;
    if (document.languageId !== 'latch') {
        vscode.window.showErrorMessage("Active file is not a Latch (.lt) document.");
        return;
    }

    // Save document before running
    if (document.isDirty) {
        document.save();
    }

    const filePath = document.fileName;
    const config = vscode.workspace.getConfiguration('latch');
    const executablePath = config.get<string>('executablePath', 'latch');
    const execMode = config.get<string>('execution.mode', 'terminal');

    const commandStr = `${executablePath} ${mode} "${filePath}"`;

    if (execMode === 'terminal') {
        let terminal = vscode.window.terminals.find(t => t.name === "Latch Execution");
        if (!terminal) {
            terminal = vscode.window.createTerminal("Latch Execution");
        }
        terminal.show(true);
        terminal.sendText(commandStr);
    } else {
        outputChannel.clear();
        outputChannel.show(true);
        outputChannel.appendLine(`Executing: ${commandStr}\n`);

        const { exec } = require('child_process');
        exec(commandStr, (err: any, stdout: string, stderr: string) => {
            if (stdout) {
                outputChannel.appendLine(stdout);
            }
            if (stderr) {
                outputChannel.appendLine(`[STDERR]\n${stderr}`);
            }
            if (err) {
                outputChannel.appendLine(`[EXIT CODE] ${err.code}`);
            } else {
                outputChannel.appendLine("\n[SUCCESS] Completed.");
            }
        });
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
