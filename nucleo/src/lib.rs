//! Núcleo do Corujão. A interface (Flutter) fala com este crate pelo `api`;
//! o resto são os módulos de verdade.
pub mod api;
pub mod biblioteca;
pub mod catalogo;
pub mod configuracao;
pub mod disco;
pub mod execucao;
mod frb_generated;
pub mod maquina;
pub mod nome;
pub mod raiz;
