const vscode = require('vscode');

let diagnosticCollection;

function activate(context) {
  diagnosticCollection = vscode.languages.createDiagnosticCollection('gala');

  context.subscriptions.push(
    vscode.commands.registerCommand('gala.check', () => checkFile()),
    vscode.commands.registerCommand('gala.run', () => runFile()),
    vscode.commands.registerCommand('gala.explain', () => explainCode()),
    vscode.commands.registerCommand('gala.showCircuit', () => showCircuit()),
    vscode.languages.registerCodeLensProvider('gala', new GalaCircuitLens())
  );

  if (vscode.workspace.getConfiguration('gala').get('lintOnSave')) {
    context.subscriptions.push(
      vscode.workspace.onDidSaveTextDocument(doc => {
        if (doc.languageId === 'gala') checkFile(doc.uri);
      })
    );
  }
}

function get_config() {
  return vscode.workspace.getConfiguration('gala');
}

async function checkFile(uri) {
  const editor = uri ? undefined : vscode.window.activeTextEditor;
  const doc = uri ? await vscode.workspace.openTextDocument(uri) : editor?.document;
  if (!doc || doc.languageId !== 'gala') return;

  const compilerPath = config().compilerPath;
  const result = await execGala(compilerPath, ['check', doc.fileName]);

  if (result !== null) {
    const diagnostics = parseDiagnostics(result.stderr || result.stdout);
    diagnosticCollection.set(doc.uri, diagnostics);
  }
}

async function runFile() {
  const editor = vscode.window.activeTextEditor;
  if (!editor || editor.document.languageId !== 'gala') return;

  const compilerPath = config().compilerPath;
  const doc = editor.document;
  await doc.save();

  const result = await execGala(compilerPath, ['run', doc.fileName]);

  if (result) {
    const channel = vscode.window.createOutputChannel('Gala');
    channel.clear();
    channel.appendLine('[Gala run]');
    channel.append(result.stdout || '(no output)');
    if (result.stderr) {
      channel.appendLine('');
      channel.appendLine('[stderr]');
      channel.append(result.stderr);
    }
    channel.show();
  }
}

async function explainCode() {
  const code = await vscode.window.showInputBox({
    prompt: 'Enter a Gala error code (e.g., E0530)',
    placeHolder: 'E0530'
  });
  if (!code) return;

  const compilerPath = config().compilerPath;
  const result = await execGala(compilerPath, ['explain', code]);

  if (result) {
    vscode.window.showInformationMessage(
      result.stdout || 'No explanation available',
      { modal: true }
    );
  }
}

function showCircuit() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return;
  const selection = editor.selection;
  const text = selection.isEmpty
    ? editor.document.getText()
    : editor.document.getText(selection);

  const panel = vscode.window.createWebviewPanel(
    'galaCircuit',
    'Gala Circuit Diagram',
    vscode.ViewColumn.Beside,
    { enableScripts: false }
  );

  panel.webview.html = `<!DOCTYPE html>
<html><head><meta charset="UTF-8">
<style>
  body { font-family: monospace; padding: 16px; background: #1e1e1e; color: #d4d4d4; }
  pre { white-space: pre-wrap; }
</style></head><body>
<h2>Circuit View</h2>
<pre>${escapeHtml(text)}</pre>
<p><em>Full circuit diagram rendering requires gala-lsp.</em></p>
</body></html>`;
}

class GalaCircuitLens {
  provideCodeLenses(document) {
    const lenses = [];
    const text = document.getText();
    const quantumFnRe = /fn\s+(\w+)\s*\([^)]*\)\s*->\s*(?:<[^>]+>\s*)?\w+\s+quantum\s*\{/g;
    let match;
    while ((match = quantumFnRe.exec(text)) !== null) {
      const line = document.positionAt(match.index).line;
      lenses.push(new vscode.CodeLens(
        new vscode.Range(line, 0, line, 0),
        { title: '🔬 Show circuit', command: 'gala.showCircuit', arguments: [match[0]] }
      ));
    }
    return lenses;
  }
}

async function execGala(binary, args) {
  const { execSync } = require('child_process');
  try {
    const output = execSync(`${binary} ${args.join(' ')}`, {
      encoding: 'utf8',
      timeout: 10000
    });
    return { stdout: output, stderr: '' };
  } catch (err) {
    if (err.code === 'ENOENT') {
      vscode.window.showErrorMessage(
        `Gala compiler not found at "${binary}". Set gala.compilerPath in settings.`
      );
      return null;
    }
    if (err.stderr) return { stdout: err.stdout || '', stderr: err.stderr };
    if (err.stdout) return { stdout: err.stdout, stderr: '' };
    return { stdout: '', stderr: err.message };
  }
}

function parseDiagnostics(text) {
  const diagnostics = [];
  for (const line of text.split('\n')) {
    const match = line.match(/^(E\d{4}):\s+(.+)$/);
    if (!match) continue;
    diagnostics.push(new vscode.Diagnostic(
      new vscode.Range(0, 0, 0, 0),
      `${match[1]}: ${match[2]}`,
      vscode.DiagnosticSeverity.Error
    ));
  }
  return diagnostics;
}

function escapeHtml(text) {
  return text.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

exports.activate = activate;