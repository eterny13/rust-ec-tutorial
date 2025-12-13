# RustでつくるスケーラブルなECバックエンド (Modular Monorepo & Clean Architecture)

このチュートリアルでは、Rustを用いてECサイト（E-commerce）のバックエンドシステムを構築します。
単なる機能実装にとどまらず、**「Effective Rust」** な記述、関数型プログラミングのエッセンス（代数型など）、**クリーンアーキテクチャ**、そしてKafkaを用いた非同期メッセージングによるスケーラビリティの確保に焦点を当てます。

## ターゲット読者
- 他の言語（Java, Go, C++など）でバックエンド開発の経験がある方
- Rustの基本構文は知っているが、実践的なアプリケーション設計や非同期処理のベストプラクティスを学びたい方

## 技術スタック
- **言語**: Rust (Edition 2021)
- **Webフレームワーク**: actix-web
- **DB**: MySQL (or MariaDB)
- **非同期ランタイム**: Tokio (actix-web依存)
- **メッセージング**: Apache Kafka
- **コンテナ**: Docker & Docker Compose

## プロジェクト構成（モノレポ + クリーンアーキテクチャ）

```
rust-practice-ec/
├── Cargo.toml                    # ワークスペース定義
├── docker-compose.yml            # インフラ（MySQL, Kafka）
└── src/
    ├── order/                    # 注文サービス
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs           # エントリーポイント
    │       ├── domain/           # ドメイン層（エンティティ、値オブジェクト）
    │       ├── service/          # サービス層（ユースケース）
    │       ├── datasource/       # データソース層（リポジトリ実装）
    │       └── controller/       # コントローラー層（HTTPハンドラ）
    ├── inventory/                # 在庫サービス
    │   └── ...（同様の構成）
    └── payment/                  # 決済サービス
        └── ...（同様の構成）
```

> [!NOTE]
> **モジュラーモノリス的アプローチ**: 各サービスは独立したクレート（パッケージ）として構成されていますが、1つのリポジトリで管理します。将来的にマイクロサービスとして分離する際にも、この構成なら容易に対応できます。

## チュートリアルの構成

### Module 1: プロジェクトのセットアップとアーキテクチャ設計
- Cargo Workspaceによるモノレポ構成
- クリーンアーキテクチャの各層の責務
- Docker Composeによるインフラ構築（MySQL, Kafka, Zookeeper）

### Module 2: ドメイン層の実装
- **Algebraic Data Types (ADTs)**: Enumを用いた無効な状態の排除
- New Type Pattern とトレイトによる振る舞いの抽象化
- ドメインエンティティとビジネスルール
- **テストコードの作成と実行**

### Module 3: データソース層の実装
- 非同期DB接続（`sqlx`）
- Connection Poolの管理とブロッキング回避
- Repositoryパターンによる永続化の抽象化
- **テストコードの作成と実行**

### Module 4: サービス層の実装
- ユースケースの実装
- トレイトを用いた依存性の注入
- エラーハンドリング
- **テストコードの作成と実行**

### Module 5: コントローラー層の実装 (Actix-web)
- リクエストハンドリングとDTO
- エラーハンドリングのミドルウェア化
- **サーバー起動の確認**
- **テストコードの作成と実行**

### Module 6: Kafkaによる非同期メッセージング
- イベント駆動アーキテクチャ（EDA）
- Producer/Consumerの実装

---

# Module 1: プロジェクトのセットアップとアーキテクチャ設計

このモジュールでは、開発環境の構築と、クリーンアーキテクチャに基づいたプロジェクト構造を作成します。

## 1.1 クリーンアーキテクチャの概要

各サービスは以下の4層で構成されます。依存の向きは **外側から内側へ一方向** です。

```
┌─────────────────────────────────────────────┐
│           Controller層 (HTTP)               │  ← 外側（フレームワーク依存）
├─────────────────────────────────────────────┤
│           Service層 (Use Cases)             │
├─────────────────────────────────────────────┤
│           Datasource層 (Repository実装)     │
├─────────────────────────────────────────────┤
│           Domain層 (Entities)               │  ← 内側（純粋なビジネスロジック）
└─────────────────────────────────────────────┘
```

### 各層の責務

| 層 | 責務 | 依存先 |
|---|---|---|
| **Domain** | エンティティ、値オブジェクト、ドメインロジック、リポジトリトレイト（インターフェース） | なし（純粋） |
| **Service** | ユースケース、ビジネスオーケストレーション | Domain層のみ |
| **Datasource** | リポジトリトレイトの具体的な実装（MySQL, Kafkaなど） | Domain層のみ |
| **Controller** | HTTPリクエスト/レスポンスの変換、ルーティング | Service層, Domain層 |

> [!TIP]
> **Effective Rust**: Domain層はフレームワーク（actix-web）や外部ライブラリ（sqlx）に依存しないように保ちます。これにより、ドメインロジックのテストが容易になり、フレームワークの変更にも影響を受けません。

## 1.2 Cargo Workspaceの構築

### Task: ワークスペースの作成

ルートの `Cargo.toml` を以下のように定義してください。

```toml
[workspace]
members = [
    "src/order",
    "src/inventory",
    "src/payment",
]
resolver = "2"

# ワークスペース全体で共有する依存関係
[workspace.dependencies]
# Web Framework
actix-web = "4"
actix-rt = "2"

# Async Runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "mysql", "macros", "chrono", "uuid"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Error Handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utility
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# Testing
mockall = "0.11"
```

> [!TIP]
> **`workspace.dependencies`** を使うことで、各サービスで同じバージョンのライブラリを使用でき、依存関係の管理が簡潔になります。各サービスの `Cargo.toml` では `{ workspace = true }` と書くだけです。

### Task: 各サービスの Cargo.toml

各サービス（例: `src/order/Cargo.toml`）は以下のように定義します。

```toml
[package]
name = "order"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "order"
path = "src/main.rs"

[dependencies]
actix-web = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
dotenv = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[dev-dependencies]
mockall = { workspace = true }
```

### Task: ディレクトリ構造の作成

各サービス内にクリーンアーキテクチャの層を作成します。

```bash
# Order サービス
mkdir -p src/order/src/{domain,service,datasource,controller}

# Inventory サービス  
mkdir -p src/inventory/src/{domain,service,datasource,controller}

# Payment サービス
mkdir -p src/payment/src/{domain,service,datasource,controller}
```

## 1.3 Docker Composeによるインフラ構築

### Task: docker-compose.yml の作成

以下の要件を満たす `docker-compose.yml` をルートディレクトリに作成してください。

1.  **MySQL 8.0**:
    - データ永続化のためにVolumeマウントを行うこと。
    - 初期データベース作成用の環境変数を設定すること。
2.  **Zookeeper & Kafka**:
    - サービス間の通信が可能になるよう、ポート設定（例: 9092）に注意してください。

**ヒント:** 各サービスで論理的に異なるデータベース（Schemas）を使用します（例: `order_db`, `inventory_db`, `payment_db`）。

```yaml
version: '3.8'
services:
  mysql:
    image: mysql:8.0
    environment:
      MYSQL_ROOT_PASSWORD: rootpassword
      MYSQL_USER: ecuser
      MYSQL_PASSWORD: ecpassword
    ports:
      - "3306:3306"
    volumes:
      - mysql_data:/var/lib/mysql
      - ./init-db.sql:/docker-entrypoint-initdb.d/init.sql
    healthcheck:
      test: ["CMD", "mysqladmin", "ping", "-h", "localhost"]
      timeout: 20s
      retries: 10

  zookeeper:
    image: confluentinc/cp-zookeeper:7.4.0
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181

  kafka:
    image: confluentinc/cp-kafka:7.4.0
    depends_on:
      - zookeeper
    ports:
      - "9092:9092"
    environment:
      KAFKA_BROKER_ID: 1
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1

volumes:
  mysql_data:
```

### Task: 初期化SQLの作成

```sql
-- init-db.sql
CREATE DATABASE IF NOT EXISTS order_db;
CREATE DATABASE IF NOT EXISTS inventory_db;
CREATE DATABASE IF NOT EXISTS payment_db;

GRANT ALL PRIVILEGES ON order_db.* TO 'ecuser'@'%';
GRANT ALL PRIVILEGES ON inventory_db.* TO 'ecuser'@'%';
GRANT ALL PRIVILEGES ON payment_db.* TO 'ecuser'@'%';
```

---

# Module 2: ドメイン層の実装

ドメイン層は最も内側の層であり、外部ライブラリに依存しない純粋なビジネスロジックを記述します。

## 2.1 ディレクトリ構成

```
src/order/src/domain/
├── mod.rs              # モジュール定義
├── order.rs            # Orderエンティティ
├── order_id.rs         # OrderId値オブジェクト
├── order_status.rs     # OrderStatus列挙型
├── order_item.rs       # OrderItemエンティティ
├── order_error.rs      # ドメインエラー
└── order_repository.rs # リポジトリトレイト（インターフェース）
```

## 2.2 New Type Pattern による型安全性

IDや数量などを単なる `String` や `i32` で扱うと、取り違えのバグが発生しやすくなります。

### Task: 値オブジェクトの作成

`src/order/src/domain/order_id.rs`:

```rust
use serde::{Deserialize, Serialize};

/// 注文を一意に識別するID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(String);

impl OrderId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for OrderId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for OrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

同様に `CustomerId`, `ProductId` なども作成してください。

> [!TIP]
> **Effective Rust**: `impl Into<String>` を使うことで、`&str` と `String` の両方を受け付けるAPIを簡潔に書けます。

## 2.3 Algebraic Data Types (ADTs) で状態を表現する

Rustの `enum` は直和型（Sum Type）であり、「無効な状態」をコンパイルレベルで排除できます。

### Task: 注文ステータスの定義

`src/order/src/domain/order_status.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// 支払い待ち
    PendingPayment,
    /// 支払い失敗（理由を含む）
    PaymentFailed(String),
    /// 支払い完了
    Paid,
    /// 発送済み（追跡番号を含む）
    Shipped { tracking_number: String },
    /// 配達完了
    Delivered,
    /// キャンセル済み
    Cancelled,
}

impl OrderStatus {
    /// この状態でアイテム追加が可能かどうか
    pub fn can_add_item(&self) -> bool {
        matches!(self, OrderStatus::PendingPayment)
    }

    /// この状態でキャンセル可能かどうか
    pub fn can_cancel(&self) -> bool {
        matches!(self, OrderStatus::PendingPayment | OrderStatus::Paid)
    }
}
```

このように、**各状態に必要なデータのみを持たせる**ことで、 `Option<String>` を多用する「nullチェック地獄」から解放されます。

## 2.4 ドメインエンティティの実装

### Task: Orderエンティティの作成

`src/order/src/domain/order.rs`:

```rust
use crate::domain::{
    CustomerId, OrderId, OrderItem, OrderStatus, OrderError
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Order {
    id: OrderId,
    customer_id: CustomerId,
    items: Vec<OrderItem>,
    status: OrderStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Order {
    /// 新しい注文を作成
    pub fn new(customer_id: CustomerId) -> Self {
        let now = Utc::now();
        Self {
            id: OrderId::generate(),
            customer_id,
            items: Vec::new(),
            status: OrderStatus::PendingPayment,
            created_at: now,
            updated_at: now,
        }
    }

    /// アイテムを追加（PendingPayment状態でのみ可能）
    pub fn add_item(&mut self, item: OrderItem) -> Result<(), OrderError> {
        if !self.status.can_add_item() {
            return Err(OrderError::InvalidStateTransition {
                current: format!("{:?}", self.status),
                action: "add_item".to_string(),
            });
        }
        self.items.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 合計金額を計算
    pub fn total_amount(&self) -> u64 {
        self.items.iter().map(|item| item.subtotal()).sum()
    }

    /// 支払い完了にする
    pub fn mark_as_paid(&mut self) -> Result<(), OrderError> {
        match &self.status {
            OrderStatus::PendingPayment => {
                self.status = OrderStatus::Paid;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(OrderError::InvalidStateTransition {
                current: format!("{:?}", self.status),
                action: "mark_as_paid".to_string(),
            }),
        }
    }

    // Getters
    pub fn id(&self) -> &OrderId { &self.id }
    pub fn customer_id(&self) -> &CustomerId { &self.customer_id }
    pub fn items(&self) -> &[OrderItem] { &self.items }
    pub fn status(&self) -> &OrderStatus { &self.status }
}
```

### Task: ドメインエラーの定義

`src/order/src/domain/order_error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrderError {
    #[error("Invalid state transition: cannot {action} when status is {current}")]
    InvalidStateTransition { current: String, action: String },

    #[error("Order not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}
```

> [!TIP]
> **Effective Rust**: `thiserror` クレートを使うと、`std::error::Error` トレイトの実装を簡潔に書けます。ドメイン層で使用するのは `thiserror` のみで、`anyhow` はアプリケーション層で使用します。

## 2.5 リポジトリトレイト（インターフェース）の定義

**重要**: リポジトリの「インターフェース（トレイト）」はドメイン層に配置し、「実装」はデータソース層に配置します。

### Task: リポジトリトレイトの作成

`src/order/src/domain/order_repository.rs`:

```rust
use async_trait::async_trait;
use crate::domain::{Order, OrderId, OrderError};

/// 注文の永続化を抽象化するトレイト
/// 具体的な実装（MySQL, PostgreSQLなど）はDatasource層で行う
#[async_trait]
pub trait OrderRepository: Send + Sync {
    /// IDで注文を検索
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, OrderError>;
    
    /// 注文を保存（新規作成または更新）
    async fn save(&self, order: &Order) -> Result<(), OrderError>;
    
    /// 顧客IDで注文一覧を取得
    async fn find_by_customer_id(&self, customer_id: &str) -> Result<Vec<Order>, OrderError>;
}
```

> [!NOTE]
> `async_trait` クレートは、トレイト内で非同期関数を定義するために必要です。`Send + Sync` を付けることで、マルチスレッド環境でも安全に使用できます。

### Step: テストコードの作成と実行

**ドメインロジックを実装したら、すぐにテストを書いて検証しましょう。**

`src/order/src/domain/order_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CustomerId, OrderItem, ProductId, OrderStatus};

    #[test]
    fn test_new_order_has_pending_payment_status() {
        let order = Order::new(CustomerId::new("customer-1"));
        
        assert!(matches!(order.status(), OrderStatus::PendingPayment));
        assert!(order.items().is_empty());
    }

    #[test]
    fn test_add_item_when_pending_payment() {
        let mut order = Order::new(CustomerId::new("customer-1"));
        let item = OrderItem::new(
            ProductId::new("product-1"),
            2,
            1000,
        );

        let result = order.add_item(item);

        assert!(result.is_ok());
        assert_eq!(order.items().len(), 1);
    }

    #[test]
    fn test_add_item_fails_when_paid() {
        let mut order = Order::new(CustomerId::new("customer-1"));
        order.mark_as_paid().unwrap(); // 状態を変更

        let item = OrderItem::new(
            ProductId::new("product-1"),
            1,
            500,
        );

        let result = order.add_item(item);

        assert!(result.is_err());
    }

    #[test]
    fn test_total_amount_calculation() {
        let mut order = Order::new(CustomerId::new("customer-1"));
        order.add_item(OrderItem::new(ProductId::new("p1"), 2, 1000)).unwrap();
        order.add_item(OrderItem::new(ProductId::new("p2"), 3, 500)).unwrap();

        // 2 * 1000 + 3 * 500 = 3500
        assert_eq!(order.total_amount(), 3500);
    }

    #[test]
    fn test_order_status_transitions() {
        let status = OrderStatus::Shipped {
            tracking_number: "TRACK123".to_string(),
        };

        match status {
            OrderStatus::Shipped { tracking_number } => {
                assert_eq!(tracking_number, "TRACK123");
            }
            _ => panic!("Expected Shipped status"),
        }
    }
}
```

#### テストの実行

```bash
# Order サービスのテストを実行
cargo test -p order

# 特定のテスト関数のみ実行
cargo test -p order test_add_item_when_pending_payment

# テストの出力を表示
cargo test -p order -- --nocapture

# 全ワークスペースのテストを実行
cargo test --workspace
```

> [!TIP]
> **Effective Rust**: テストは `#[cfg(test)]` で囲むのがRustの慣習です。本番ビルドには含まれません。AAA（Arrange-Act-Assert）パターンでテストを構造化すると読みやすくなります。

---

# Module 3: データソース層の実装

データソース層では、ドメイン層で定義したリポジトリトレイトを具体的な技術（MySQL）を使って実装します。

## 3.1 ディレクトリ構成

```
src/order/src/datasource/
├── mod.rs
├── order_repository_db.rs    # MySQLを使ったリポジトリ実装
└── order_record.rs           # DBレコードとドメインモデルの変換
```

## 3.2 sqlx のセットアップ

`sqlx` は純粋な非同期ドライバであり、**コンパイル時のSQL検証機能**が強力です。

### Task: Connection Pool の管理

`src/order/src/datasource/mod.rs`:

```rust
use sqlx::mysql::MySqlPoolOptions;
use sqlx::MySqlPool;

pub mod order_repository_db;
pub mod order_record;

/// Connection Pool を作成
pub async fn create_pool(database_url: &str) -> Result<MySqlPool, sqlx::Error> {
    MySqlPoolOptions::new()
        .max_connections(20)  // 負荷に応じて調整
        .connect(database_url)
        .await
}
```

> [!WARNING]
> **Blocking vs Non-Blocking**: DBクエリはI/Oバウンドな操作です。同期的なドライバを使うとクエリ待ちでスレッドがブロックされ、性能が劣化します。`sqlx` のような非同期ドライバを使うことで、少数のスレッドで数千のリクエストを捌けます。

## 3.3 DBレコードとドメインモデルの変換

### Task: レコード構造体の定義

`src/order/src/datasource/order_record.rs`:

```rust
use sqlx::FromRow;
use chrono::{DateTime, Utc};

/// DBから取得した行をマッピングするための構造体
#[derive(Debug, FromRow)]
pub struct OrderRecord {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_amount: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
pub struct OrderItemRecord {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub quantity: i32,
    pub unit_price: i64,
}
```

## 3.4 リポジトリの実装

### Task: MySQLリポジトリの実装

`src/order/src/datasource/order_repository_db.rs`:

```rust
use async_trait::async_trait;
use sqlx::MySqlPool;
use crate::domain::{Order, OrderId, OrderError, OrderRepository};
use super::order_record::OrderRecord;

pub struct OrderRepositoryDb {
    pool: MySqlPool,
}

impl OrderRepositoryDb {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderRepository for OrderRepositoryDb {
    async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, OrderError> {
        let record = sqlx::query_as::<_, OrderRecord>(
            r#"
            SELECT id, customer_id, status, total_amount, created_at, updated_at
            FROM orders
            WHERE id = ?
            "#
        )
        .bind(id.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OrderError::Infrastructure(e.to_string()))?;

        match record {
            Some(rec) => {
                // TODO: レコードからドメインモデルへの変換
                todo!("Convert OrderRecord to Order")
            }
            None => Ok(None),
        }
    }

    async fn save(&self, order: &Order) -> Result<(), OrderError> {
        sqlx::query(
            r#"
            INSERT INTO orders (id, customer_id, status, total_amount, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                status = VALUES(status),
                total_amount = VALUES(total_amount),
                updated_at = VALUES(updated_at)
            "#
        )
        .bind(order.id().as_str())
        .bind(order.customer_id().as_str())
        .bind(format!("{:?}", order.status()))
        .bind(order.total_amount() as i64)
        .bind(order.created_at())
        .bind(order.updated_at())
        .execute(&self.pool)
        .await
        .map_err(|e| OrderError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn find_by_customer_id(&self, customer_id: &str) -> Result<Vec<Order>, OrderError> {
        // TODO: 実装
        todo!()
    }
}
```

### Step: テストコードの作成と実行

リポジトリ層のテストでは、実際のDBを使った統合テストが有効です。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    async fn get_test_pool() -> MySqlPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set");
        MySqlPool::connect(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_save_and_find_order() {
        // Arrange
        let pool = get_test_pool().await;
        let repo = OrderRepositoryDb::new(pool);
        
        let order = Order::new(CustomerId::new("customer-1"));

        // Act
        repo.save(&order).await.expect("Failed to save order");
        let found = repo.find_by_id(order.id()).await.expect("Failed to find order");

        // Assert
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), order.id());
    }
}
```

#### テストの実行

```bash
# Docker ComposeでテストDBを起動
docker compose up -d mysql

# 環境変数を設定してテスト実行
TEST_DATABASE_URL="mysql://ecuser:ecpassword@localhost:3306/order_db" cargo test -p order

# DB統合テストは直列実行推奨（競合回避）
cargo test -p order -- --test-threads=1
```

> [!WARNING]
> **統合テストの注意点**: DBを使うテストは互いに干渉しないよう、テストごとにユニークなIDを使用するか、テスト後にクリーンアップしてください。

---

# Module 4: サービス層の実装

サービス層では、ユースケース（ビジネスロジックのオーケストレーション）を実装します。

## 4.1 ディレクトリ構成

```
src/order/src/service/
├── mod.rs
├── order_service.rs       # 注文のユースケース
└── order_service_error.rs # サービス層のエラー
```

## 4.2 依存性の注入（DI）

サービス層はリポジトリに依存しますが、**具体的な実装（MySQL）ではなく、トレイト（インターフェース）に依存**します。

### Task: サービスの実装

`src/order/src/service/order_service.rs`:

```rust
use std::sync::Arc;
use crate::domain::{
    CustomerId, Order, OrderError, OrderId, OrderItem, OrderRepository,
};

pub struct OrderService<R: OrderRepository> {
    repository: Arc<R>,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 新しい注文を作成
    pub async fn create_order(&self, customer_id: CustomerId) -> Result<Order, OrderError> {
        let order = Order::new(customer_id);
        self.repository.save(&order).await?;
        Ok(order)
    }

    /// 注文にアイテムを追加
    pub async fn add_item_to_order(
        &self,
        order_id: &OrderId,
        item: OrderItem,
    ) -> Result<Order, OrderError> {
        let mut order = self.repository
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| OrderError::NotFound(order_id.to_string()))?;

        order.add_item(item)?;
        self.repository.save(&order).await?;
        
        Ok(order)
    }

    /// 注文を取得
    pub async fn get_order(&self, order_id: &OrderId) -> Result<Option<Order>, OrderError> {
        self.repository.find_by_id(order_id).await
    }
}
```

> [!TIP]
> **Effective Rust**: ジェネリクス `<R: OrderRepository>` を使うことで、テスト時にはモックリポジトリを、本番時にはMySQLリポジトリを注入できます。これが「依存性逆転の原則」のRust流実装です。

### Step: テストコードの作成と実行（モックを使用）

サービス層のテストでは、`mockall` を使ってリポジトリをモックします。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use crate::domain::OrderRepository;

    // モックの自動生成
    mockall::mock! {
        pub OrderRepo {}
        
        #[async_trait::async_trait]
        impl OrderRepository for OrderRepo {
            async fn find_by_id(&self, id: &OrderId) -> Result<Option<Order>, OrderError>;
            async fn save(&self, order: &Order) -> Result<(), OrderError>;
            async fn find_by_customer_id(&self, customer_id: &str) -> Result<Vec<Order>, OrderError>;
        }
    }

    #[tokio::test]
    async fn test_create_order_success() {
        // Arrange
        let mut mock_repo = MockOrderRepo::new();
        mock_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let service = OrderService::new(Arc::new(mock_repo));

        // Act
        let result = service.create_order(CustomerId::new("customer-1")).await;

        // Assert
        assert!(result.is_ok());
        let order = result.unwrap();
        assert!(matches!(order.status(), OrderStatus::PendingPayment));
    }

    #[tokio::test]
    async fn test_add_item_to_non_existent_order() {
        // Arrange
        let mut mock_repo = MockOrderRepo::new();
        mock_repo
            .expect_find_by_id()
            .returning(|_| Ok(None)); // 注文が見つからない

        let service = OrderService::new(Arc::new(mock_repo));
        let order_id = OrderId::new("non-existent");
        let item = OrderItem::new(ProductId::new("p1"), 1, 1000);

        // Act
        let result = service.add_item_to_order(&order_id, item).await;

        // Assert
        assert!(matches!(result, Err(OrderError::NotFound(_))));
    }
}
```

#### テストの実行

```bash
# サービス層のテストを実行
cargo test -p order

# 特定のテストのみ
cargo test -p order test_create_order_success
```

---

# Module 5: コントローラー層の実装 (Actix-web)

コントローラー層では、HTTPリクエスト/レスポンスの変換とルーティングを担当します。

## 5.1 ディレクトリ構成

```
src/order/src/controller/
├── mod.rs
├── order_controller.rs    # HTTPハンドラ
├── request/               # リクエストDTO
│   ├── mod.rs
│   └── order_request.rs
└── response/              # レスポンスDTO
    ├── mod.rs
    └── order_response.rs
```

## 5.2 DTO (Data Transfer Object) の定義

APIの入力/出力はドメインモデルを直接使わず、DTOを定義します。

### Task: リクエストDTOの作成

`src/order/src/controller/request/order_request.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AddItemRequest {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price: u64,
}
```

### Task: レスポンスDTOの作成

`src/order/src/controller/response/order_response.rs`:

```rust
use serde::Serialize;
use crate::domain::Order;

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub customer_id: String,
    pub status: String,
    pub total_amount: u64,
    pub items: Vec<OrderItemResponse>,
}

#[derive(Debug, Serialize)]
pub struct OrderItemResponse {
    pub product_id: String,
    pub quantity: u32,
    pub unit_price: u64,
}

impl From<&Order> for OrderResponse {
    fn from(order: &Order) -> Self {
        Self {
            id: order.id().to_string(),
            customer_id: order.customer_id().to_string(),
            status: format!("{:?}", order.status()),
            total_amount: order.total_amount(),
            items: order.items().iter().map(|i| OrderItemResponse {
                product_id: i.product_id().to_string(),
                quantity: i.quantity(),
                unit_price: i.unit_price(),
            }).collect(),
        }
    }
}
```

## 5.3 ハンドラの実装

### Task: コントローラーの実装

`src/order/src/controller/order_controller.rs`:

```rust
use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use crate::domain::OrderRepository;
use crate::service::OrderService;
use super::request::{CreateOrderRequest, AddItemRequest};
use super::response::OrderResponse;

/// 注文を作成
pub async fn create_order<R: OrderRepository + 'static>(
    service: web::Data<Arc<OrderService<R>>>,
    body: web::Json<CreateOrderRequest>,
) -> Result<HttpResponse> {
    let customer_id = body.customer_id.clone().into();
    
    match service.create_order(customer_id).await {
        Ok(order) => {
            let response: OrderResponse = (&order).into();
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(format!("{}", e)))
        }
    }
}

/// 注文を取得
pub async fn get_order<R: OrderRepository + 'static>(
    service: web::Data<Arc<OrderService<R>>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let order_id = path.into_inner().into();
    
    match service.get_order(&order_id).await {
        Ok(Some(order)) => {
            let response: OrderResponse = (&order).into();
            Ok(HttpResponse::Ok().json(response))
        }
        Ok(None) => Ok(HttpResponse::NotFound().finish()),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("{}", e))),
    }
}
```

## 5.4 メインエントリーポイント

### Task: main.rs の実装

`src/order/src/main.rs`:

```rust
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

mod domain;
mod service;
mod datasource;
mod controller;

use datasource::{create_pool, order_repository_db::OrderRepositoryDb};
use service::OrderService;
use controller::order_controller;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 環境変数の読み込み
    dotenv::dotenv().ok();
    
    // ロガーの初期化
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Connection Pool の作成
    let pool = create_pool(&database_url)
        .await
        .expect("Failed to create pool");

    // 依存関係の組み立て（DI）
    let repository = Arc::new(OrderRepositoryDb::new(pool));
    let service = Arc::new(OrderService::new(repository));

    tracing::info!("Starting Order Service on 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(service.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/orders", web::post().to(order_controller::create_order::<OrderRepositoryDb>))
                    .route("/orders/{id}", web::get().to(order_controller::get_order::<OrderRepositoryDb>))
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### Step: サーバー起動の確認

**Controller実装後は、必ずサーバーが正常に起動するか確認しましょう。**

#### 1. 環境変数の設定

プロジェクトルートに `.env` ファイルを作成します。

```bash
# .env
DATABASE_URL=mysql://ecuser:ecpassword@localhost:3306/order_db
RUST_LOG=info
```

#### 2. 依存サービスの起動

```bash
# MySQLを起動
docker compose up -d mysql

# MySQLが起動完了するまで待機
docker compose logs -f mysql
```

#### 3. サーバーの起動

```bash
# Order Serviceを起動
cargo run -p order
```

期待される出力:
```
Starting Order Service on 0.0.0.0:8080
```

#### 4. ヘルスチェック

別のターミナルからリクエストを送信して動作確認します。

```bash
# 注文を作成
curl -X POST http://localhost:8080/api/v1/orders \
  -H "Content-Type: application/json" \
  -d '{"customer_id": "customer-1"}'

# 注文を取得
curl http://localhost:8080/api/v1/orders/{order_id}
```

#### 5. トラブルシューティング

起動に失敗した場合のチェックポイント:

- **DB接続エラー**: `DATABASE_URL` が正しいか、MySQLが起動しているか確認
- **ポート競合**: 8080ポートが他のプロセスで使用されていないか確認
- **コンパイルエラー**: `cargo check -p order` でエラーを確認

```bash
# ポートの使用状況を確認
lsof -i :8080

# 詳細なログを有効化
RUST_LOG=debug cargo run -p order
```

> [!TIP]
> **Effective Rust**: 開発中は `cargo watch` を使うと、ファイル変更時に自動で再コンパイル・再起動してくれます。`cargo install cargo-watch` 後、`cargo watch -x 'run -p order'` で実行できます。

### Step: Controller層のテストコード作成と実行

Actix-webにはテスト用のユーティリティが用意されています。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App, web};

    #[actix_web::test]
    async fn test_create_order_endpoint() {
        // TODO: モックサービスを使ったテスト
        let app = test::init_service(
            App::new()
                .route("/api/v1/orders", web::post().to(|| async {
                    HttpResponse::Created().json(serde_json::json!({
                        "id": "order-1",
                        "customer_id": "customer-1",
                        "status": "PendingPayment"
                    }))
                }))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/v1/orders")
            .set_json(serde_json::json!({
                "customer_id": "customer-1"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }
}
```

#### テストの実行

```bash
# Controller層のテストを実行
cargo test -p order

# すべてのワークスペースメンバーのテストを実行
cargo test --workspace
```

---

# Module 6: Kafkaによる非同期メッセージング

サービス間の連携には、HTTP（同期）ではなくメッセージング（非同期）を優先します。

## 6.1 イベントの定義

各サービスで発生するドメインイベントを定義します。

`src/order/src/domain/order_event.rs`:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    OrderCreated {
        order_id: String,
        customer_id: String,
        created_at: DateTime<Utc>,
    },
    OrderPaid {
        order_id: String,
        total_amount: u64,
        paid_at: DateTime<Utc>,
    },
}
```

## 6.2 Kafka Producer の実装

`src/order/src/datasource/kafka_publisher.rs`:

```rust
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::time::Duration;
use crate::domain::OrderEvent;

pub struct KafkaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventPublisher {
    pub fn new(brokers: &str, topic: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        Self {
            producer,
            topic: topic.to_owned(),
        }
    }

    pub async fn publish(&self, event: &OrderEvent) -> Result<(), String> {
        let payload = serde_json::to_string(event)
            .map_err(|e| e.to_string())?;
        
        let key = match event {
            OrderEvent::OrderCreated { order_id, .. } => order_id.clone(),
            OrderEvent::OrderPaid { order_id, .. } => order_id.clone(),
        };

        let record = FutureRecord::to(&self.topic)
            .payload(&payload)
            .key(&key);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| e.to_string())?;

        Ok(())
    }
}
```

---

# おわりに

このチュートリアルを通して、クリーンアーキテクチャに基づいたRust ECバックエンドのプロトタイプを構築しました。

## サマリー

| 層 | 責務 | テスト方法 |
|---|---|---|
| **Domain** | エンティティ、ビジネスルール | 純粋なユニットテスト |
| **Service** | ユースケース | モックを使ったユニットテスト |
| **Datasource** | DB操作、外部API | 統合テスト |
| **Controller** | HTTP変換 | Actix-webのテストユーティリティ |

## 次のステップ

1. **分散トレーシング**: `tracing`, `opentelemetry` を導入
2. **Sagaパターン**: 分散トランザクションの実装
3. **デプロイ**: `cargo build --release` で最適化されたバイナリを作成

Rustでの開発はコンパイラとの対話です。エラーメッセージは厳格ですが、それを乗り越えた先には堅牢で高速なシステムが待っています。

Happy Coding! 🦀
