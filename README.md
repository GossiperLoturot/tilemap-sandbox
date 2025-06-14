# tilemap-sandbox

![image](https://github.com/user-attachments/assets/2bbd5937-ccb6-4263-9c7d-7d4177f581b5)

## 概要

**tilemap-sandbox** は、Rust と Godot ゲームエンジンを用いて構築された、タイルベースの2Dゲーム開発フレームワークです。本プロジェクトは、動的に編集可能なタイル、ブロック、エンティティを備えた2Dタイルベースのワールドを構築することを目的としています。

## 特徴

主な特徴は以下のようになります。

* **関心の分離** : 自由度が高く柔軟性を備えた Godot ゲームエンジン と 高い実行効率と型安全性を持つ Rust による効果的な設計
* **効率的なデータ管理** : 空間インデクスを用いたタイル、ブロック、エンティティの効率的な管理が可能
* **柔軟な拡張性** : 「プロシージャルなワールド生成」「プレイヤーの移動や動物の AI などの動作制御」などのイベントループを高い自由度で管理可能

## 拡張

拡張が可能な要素は以下のようになります。

* 新しいタイル、ブロック、エンティティ、アイテム
* 新しいイベントループ
* 新しい Godot と Rust の API エンドポイント

## プロジェクト構成と拡張

本プロジェクトは、1つの Godot プロジェクトと2つの Rust クレートから構成されています。

* `/`：描画や入力イベントの処理を行い、アプリケーションのエントリポイントとなる Godot プロジェクト。
* `/native-core`：データフローやビューなどのコアシステムを含む。
* `/native-main`：ユーザー定義の派生機能の実装と、Godot エンジンに対する API の公開を担う。
