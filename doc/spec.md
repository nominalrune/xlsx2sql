# xlsx2sql

## About
xlsx2sql is a tool to convert xlsx to sql.
シート名がテーブル名、1行目がカラム名、2行目以降がデータであると仮定して、レコードをINSERTするSQLに変換します


## How it works

1. xlsxを標準入力から取得
   1. ドラッグアンドドロップや、-fオプションで指定されたxlsxファイルを受け取ります
2. xlsxをパースして変数に格納
   1. xlsxから、各シートのシート名と、カラム名の配列、レコードの配列を取得します
3. SQLを作成
   1. シート名とカラム名、レコードデータに基づき、SQL文を作成します
4. SQLを保存

## sample
### input
sheet name01:
businesses
sheet data01:
id,enterprise_number,registration_status_id,company_number,xard_business_identifier,payment_method_id,card_issuance_limit_coefficient,registration_status_changed_at,disastered_at,lseg_key,lseg_note,lseg_risk_check_result,yayoi_bns_grp_identifier,X_ユーザーID?
1,9700001,2,9700000000001 ,97businessid001,1,,,,123456789012345,0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをん???0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをん???0123456789A,緑,,97userid001
2,9700002,1,9700000000002 ,97businessid002,1,,,,,,,,97userid002

sheet name02:
cards
sheet data02:
business_id,name
1,業務用
2,日常用

### output
```sql
INSERT INTO `businesses` (`id`, `registration_status_id`, `payment_method_id`, `xard_business_identifier`, `yayoi_bns_grp_identifier`, `enterprise_number`, `card_issuance_limit_coefficient`, `registration_status_changed_at`, `disastered_at`, `created_at`, `updated_at`, `deleted_at`, `lseg_key`, `lseg_note`, `lseg_risk_check_result`) VALUES
(1,2,1,'97businessid001','a','9700001',1.5,'2025-07-25 00:00:00.000000',NULL,'2025-07-25 00:00:00.000000','2025-07-25 00:00:00.000000',NULL,'123456789012345','0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをんゔゕゖ0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞただちぢっつづてでとどなにぬねのはばぱひびぴふぶぷへべぺほぼぽまみむめもゃやゅゆょよらりるれろゎわゐゑをんゔゕゖ0123456789A',3),
(2,1,1,'97businessid002','a','9700002',1.5,'2025-07-25 00:00:00.000000',NULL,'2025-07-25 00:00:00.000000','2025-07-25 00:00:00.000000',NULL,NULL,NULL,NULL);

INSERT INTO `cards` (`id`, `name`) VALUES
(1,'業務用'),
(2,'日常用');
```