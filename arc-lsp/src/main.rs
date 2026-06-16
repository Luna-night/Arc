use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

// 【修复 1 & 4】正确导入 arc-core 的内容，以及 logos 和 chumsky 的 Trait
use arc_core::{parser, Token};
use logos::Logos;
use chumsky::Parser;

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    // 【修复 3】实现缺失的 shutdown 方法
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

        async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.last() {
            let source = &change.text;
            
            // 【修复 1】使用 spanned() 获取每个 Token 的字节偏移量 (Range<usize>)
            // 使用 filter_map 过滤掉无法识别的字符 (Err)，只保留合法的 Token
            let tokens: Vec<(Token, std::ops::Range<usize>)> = Token::lexer(source)
                .spanned()
                .filter_map(|(t, span)| t.ok().map(|t| (t, span)))
                .collect();

            // 【修复 2】构建 chumsky 专属的 Stream，传入 EOI (End of Input) 的 span
            let eoi = source.len()..source.len();
            let stream = chumsky::Stream::from_iter(eoi, tokens.into_iter());
            
            // 【修复 3】将构建好的 stream 传给 parser
            let diagnostics = match parser().parse(stream) {
                Ok(_) => vec![], // 解析成功，没有错误
                Err(e) => {
                    // 将 chumsky 的错误转换为 LSP 的 Diagnostic 格式
                    e.into_iter().map(|err| {
                        let span = err.span();
                        // 辅助函数：将字节偏移量转换为 LSP 需要的行列号
                        let start_pos = offset_to_position(span.start, source);
                        let end_pos = offset_to_position(span.end, source);
                        
                        Diagnostic {
                            range: Range {
                                start: start_pos,
                                end: end_pos,
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("{}", err),
                            ..Default::default()
                        }
                    }).collect()
                }
            };

            // 将诊断信息发送给 VS Code，显示红色波浪线
            self.client
                .publish_diagnostics(params.text_document.uri, diagnostics, None)
                .await;
        }
    }
}

// 辅助函数：将字符偏移量 (offset) 转换为 LSP 的行列号 (Position)
fn offset_to_position(offset: usize, source: &str) -> Position {
    let mut line = 0;
    let mut col = 0;
    for (i, c) in source.char_indices() {
        if i == offset {
            return Position { line: line as u32, character: col as u32 };
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position { line: line as u32, character: col as u32 }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}