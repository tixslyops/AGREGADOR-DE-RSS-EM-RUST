# Agregador de RSS e Atom em Rust

Projeto prático desenvolvido para a disciplina do 3º ano do curso de Engenharia da Computação. O sistema consiste em um agregador de notícias de alta performance capaz de cadastrar, buscar, parsear e persistir artigos provenientes de feeds nos formatos RSS 2.0 e Atom.

Letícia Reis - 660308

 - Descrição do Projeto

A aplicação permite que usuários cadastrem URLs de fontes RSS/Atom para centralizar a leitura de artigos. O sistema realiza atualizações de forma concorrente e eficiente, armazenando localmente as informações coletadas para consulta offline, marcação de artigos lidos e salvamento de favoritos.

 - Principais Funcionalidades

Gerenciamento de Feeds: Cadastro, listagem, remoção e atualização individual ou em lote de feeds.

Parsing Concorrente: Suporte a RSS 2.0 e Atom com controle de requisições simultâneas.

Atualização Eficiente: Uso de cabeçalhos HTTP condicionais (ETag e Last-Modified) para evitar downloads desnecessários de dados não alterados.

Persistência Local: Salvamento permanente de dados (feeds, artigos e estados) em banco de dados local.

Gerenciamento de Artigos: Filtros por lidos/não lidos, favoritos, busca por palavras-chave e filtro por feed.

Deduplicação Inteligente: Mecanismo automático para evitar duplicidade de artigos armazenados.

Tratamento de Erros: Resiliência contra timeouts, falhas de conexão, XML malformado e retornos HTTP de erro.

Arquitetura do Sistema

O projeto adota uma arquitetura em camadas modular com separação clara de responsabilidades:

- Interface do Usuário (CLI / TUI / REST API)
            
-  Aplicação / Regras de Negócio  
   
- HTTP Client (Reqwest + Tokio)
 
- Persistência (SQLite / SQLx)

- Feed Parser (RSS & Atom Parsing)

- Deduplicação e Cache Conditional


- Estrutura do Repositório

.
1 src/
2 main.rs          # Ponto de entrada da aplicação
3 client.rs        # Cliente HTTP com suporte a requisições condicionais
4 parser.rs        # Módulo de parsing de feeds RSS 2.0 e Atom
5 db.rs            # Camada de persistência e consultas SQL
6 models.rs        # Definição de estruturas (Feed, Article, etc.)
6.1 service.rs       # Lógica de controle de concorrência e deduplicação
7 tests/               # Testes integrados e unitários
8 Cargo.toml           # Dependências e configurações da build
9 README.md            # Documentação do projeto


- Decisões Técnicas

Linguagem e Runtime: Rust com a biblioteca tokio para manipulação de I/O assíncrono e execução concorrente das buscas de feeds.

Requisições HTTP: Utilização do reqwest integrado ao suporte de cabeçalhos HTTP condicionais para otimização de banda.

Parsing XML: Bibliotecas rss e atom_syndication para abstração e validação de estruturas dos feeds.

Deduplicação de Artigos: Estratégia baseada na verificação combinada da GUID/ID original do artigo ou, na ausência desta, no hash SHA-256 do par (Link + Data de Publicação).

Persistência: Uso de banco de dados relacional leve (SQLite via sqlx) para garantir que os dados continuem disponíveis após o encerramento do programa.

Controle de Concorrência: Uso de semáforos (tokio::sync::Semaphore) para limitar a quantidade máxima de requisições concorrentes, prevenindo sobrecarga de rede.

Instruções de Compilação e Execução

Pré-requisitos

Rust e Cargo (versão 1.70 ou superior) instalados via rustup.

Passo a Passo

Clone o repositório:

git clone https://github.com/tixslyops/AGREGADOR-DE-RSS-EM-RUST.git
cd AGREGADOR-DE-RSS-EM-RUST


Compile o projeto:

cargo build --release


Execute a aplicação:

cargo run


Executar a suíte de testes automatizados:

cargo test


Funcionalidades Adicionais Implementadas

Importação e exportação de arquivos no formato OPML.

Sistema de tags/categorias customizadas para organização de feeds.

Estatísticas básicas de leitura (quantidade de artigos lidos por semana, feeds mais ativos).

- Limitações Conhecidas

Feeds que requerem autenticação por Cookies ou OAuth2 não são suportados nesta versão.

Alguns feeds com formatos XML fora do padrão estrito do RSS 2.0/Atom podem falhar no parsing e ser marcados como inválidos.

- Uso de Inteligência Artificial

Em conformidade com as orientações do projeto, ferramentas de IA gerativa foram utilizadas durante o desenvolvimento para:

Auxiliar no desenho inicial das estruturas de dados em Rust.

Auxiliar no diagnóstico e depuração de erros de lifetime do compilador do Rust.

Sugerir casos de teste unitários para o módulo de parsing XML e deduplicação.

Todo o código gerado ou sugerido por IA foi revisado, testado e compreendido.

- Licença

Este projeto está sob a licença MIT.
