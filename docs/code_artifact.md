# NotificationFunction 単体テスト関数一覧（全 9 クラス）

本リストは、C1（分岐）網羅率 100%・ループ境界（0回・複数回・最大回数）・全例外（`catch`）パス網羅の基準に基づき作成したテスト関数一覧です。

| 所在的 Class | 建议测试函数名 | 覆盖分支 / 路径 |
| :--- | :--- | :--- |
| **DeleteDetectLogTests** | `DeleteLog_ログリスト0件` | `Parallel.ForEach` 循环 0 次（空列表边界） |
| **DeleteDetectLogTests** | `DeleteLog_前回失敗ファイルなし_削除実行` | `Parallel.ForEach` 循环 1 次 ＋ `if (!IsExists)` 为 True 分支 |
| **DeleteDetectLogTests** | `DeleteLog_前回失敗ファイルあり_削除スキップ` | `Parallel.ForEach` 循环 1 次 ＋ `if (!IsExists)` 为 False 分支 |
| **DeleteDetectLogTests** | `DeleteLog_複数件混在_対象のみ削除` | `Parallel.ForEach` 循环多次（True / False 混合） |
| **DeleteDetectLogTests** | `DeleteLog_ログリスト最大件数` | `Parallel.ForEach` 循环最大次数边界 |
| **DetectLogicTests** | `RunDetectLogic_開始時閾値超過_処理中断` | L95 `if (Threshold < TotalMinutes)` 为 True |
| **DetectLogicTests** | `RunDetectLogic_前回失敗ファイルあり_情報取得` | L108 `if (IsExists)` 为 True |
| **DetectLogicTests** | `RunDetectLogic_前回失敗ファイル確認例外_エラー出力` | L115 `catch (Exception)` |
| **DetectLogicTests** | `RunDetectLogic_ログ内容取得例外_エラー出力` | L131 `catch (Exception)` |
| **DetectLogicTests** | `RunDetectLogic_ログ内容0件_削除のみ実行` | L144 `foreach (contentsList)` 循环 0 次 |
| **DetectLogicTests** | `RunDetectLogic_ログ内容最大件数_正常終了` | L144 `foreach (contentsList)` 循环最大次数边界 |
| **DetectLogicTests** | `RunDetectLogic_処理済み行スキップ_判定対象外` | L148 `if (line < errorLine)` 为 True |
| **DetectLogicTests** | `RunDetectLogic_1000行毎閾値超過_例外スロー` | L160 `if (line % 1000 == 0)` True ＋ L163 `if (Threshold < TotalMinutes)` True |
| **DetectLogicTests** | `RunDetectLogic_1000行毎閾値以内_処理継続` | L160 `if (line % 1000 == 0)` True ＋ L163 `if` False |
| **DetectLogicTests** | `RunDetectLogic_通知条件不一致_判定スキップ` | L174 `if (judgmentList.Count > 0)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_システム操作ID登録済み_非通知` | L182 `is not null` True ＋ L185 `foreach` 多次 ＋ L188 `Equals` True |
| **DetectLogicTests** | `RunDetectLogic_システム操作ID不一致_通知対象継続` | L185 `foreach` 走完 ＋ L188 `Equals` 全 False |
| **DetectLogicTests** | `RunDetectLogic_システム操作ID0件_通知対象継続` | L185 `foreach (SystemOperationId)` 循环 0 次 |
| **DetectLogicTests** | `RunDetectLogic_システム操作ID未設定_通知対象継続` | L182 `if (SystemOperationId is not null)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_前回通知フラグTrue_操作判定スキップ` | L198 `if (!notificationFlag)` 为 False（走 L228 `else`） |
| **DetectLogicTests** | `RunDetectLogic_CosmosDB操作_操作判定スキップ` | L201 `if (cosmosdb != ResourceType)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_操作判定結果0件_後続スキップ` | L214 `if (judgmentList.Count > 0)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_CosmosDB_TriggerFunctionsアクセス_非通知` | L216 `cosmosdb == ResourceType` True ＋ L220 `Contains("Trigger-Functions")` True |
| **DetectLogicTests** | `RunDetectLogic_CosmosDB_通常アクセス_通知対象` | L216 `cosmosdb == ResourceType` True ＋ L220 `Contains("Trigger-Functions")` False |
| **DetectLogicTests** | `RunDetectLogic_判定処理例外_エラーファイル出力なし` | L242 `catch (Exception)` |
| **DetectLogicTests** | `RunDetectLogic_CosmosDBコンテナ該当_UPN変換実行` | L271 `if (Contains && UserId != "")` 为 True |
| **DetectLogicTests** | `RunDetectLogic_CosmosDBコンテナ以外_UPN変換スキップ` | L271 `if (Contains && UserId != "")` 为 False |
| **DetectLogicTests** | `RunDetectLogic_通知対象外_アラート発報スキップ` | L277 `if (judgmentResult.IsAlert)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_ユーザーID空_Unknownでアラート発報` | L280 三元运算符 False ＋ L286 `IsDuplicateAlert` False |
| **DetectLogicTests** | `RunDetectLogic_重複アラートあり_発報スキップ` | L286 `if (IsDuplicateAlert)` 为 True |
| **DetectLogicTests** | `RunDetectLogic_アラート処理閾値超過_エラーファイル出力` | L263 `if (Threshold < TotalMinutes)` 为 True |
| **DetectLogicTests** | `RunDetectLogic_アラート送信例外_エラーファイル出力` | L300 `catch (Exception)` |
| **DetectLogicTests** | `RunDetectLogic_検知履歴登録例外_エラーファイル出力` | L318 `catch (Exception)` |
| **DetectLogicTests** | `RunDetectLogic_通知対象複数件_全件正常処理` | L252 `foreach (judgmentResults)` 循环多次 / 最大次数 |
| **DetectLogicTests** | `RunDetectLogic_前回失敗ファイルあり_削除実行` | L333 `if (errorLine != -1)` 为 True |
| **DetectLogicTests** | `RunDetectLogic_前回失敗ファイルなし_削除スキップ` | L333 `if (errorLine != -1)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_失敗ファイル削除例外_エラー出力` | L339 `catch (Exception)` |
| **DetectLogicTests** | `RunDetectLogic_通知条件リスト0件_判定スキップ` | `DetectByConditionsList` L370 `foreach` 循环 0 次 |
| **DetectLogicTests** | `RunDetectLogic_通知条件リスト最大件数_正常終了` | `DetectByConditionsList` L370 `foreach` 循环最大次数 |
| **DetectLogicTests** | `RunDetectLogic_ストレージ対象一致_判定リスト追加` | `DetectByConditionsList` L382 `blob/file/table` ＋ L391 `CheckStorageAccount` True |
| **DetectLogicTests** | `RunDetectLogic_ストレージ対象不一致_判定リスト追加なし` | `DetectByConditionsList` L382 `blob/file/table` ＋ L391 `CheckStorageAccount` False |
| **DetectLogicTests** | `RunDetectLogic_SQLサーバー名不一致_次条件へ` | `DetectByConditionsList` L404 `sqldb/sqlmi` ＋ L411 `ServerName` 校验 False |
| **DetectLogicTests** | `RunDetectLogic_SQLデータベース名不一致_次条件へ` | `DetectByConditionsList` L404 `sqldb/sqlmi` ＋ L419 `DBName` 校验 False |
| **DetectLogicTests** | `RunDetectLogic_SQL操作内容あり_判定リスト追加` | `DetectByConditionsList` L404 `sqldb/sqlmi` ＋ L428 `!IsNullOrEmpty(operation)` True |
| **DetectLogicTests** | `RunDetectLogic_SQL操作内容なし_判定リスト追加なし` | `DetectByConditionsList` L404 `sqldb/sqlmi` ＋ L428 `!IsNullOrEmpty(operation)` False |
| **DetectLogicTests** | `RunDetectLogic_CosmosDBサーバー名不一致_次条件へ` | `DetectByConditionsList` L441 `cosmosdb` ＋ L446 `ServerName` 校验 False |
| **DetectLogicTests** | `RunDetectLogic_CosmosDBデータベース名一致_判定リスト追加` | `DetectByConditionsList` L441 `cosmosdb` ＋ L454 `DBName` 校验 True |
| **DetectLogicTests** | `RunDetectLogic_CosmosDBデータベース名不一致_判定リスト追加なし` | `DetectByConditionsList` L441 `cosmosdb` ＋ L454 `DBName` 校验 False |
| **DetectLogicTests** | `RunDetectLogic_操作方法リソース名一致_操作一致_通知対象` | `DetectByOperationMethod` L493 `resourceMatch.Count > 0` True ＋ L496 `Any` True |
| **DetectLogicTests** | `RunDetectLogic_操作方法リソース名一致_操作不一致_非通知` | `DetectByOperationMethod` L493 `resourceMatch.Count > 0` True ＋ L496 `Any` False |
| **DetectLogicTests** | `RunDetectLogic_操作方法システムコード一致_操作一致_通知対象` | `DetectByOperationMethod` L493 False ＋ L502 `sysCodeMatch.Count > 0` True ＋ L505 `Any` True |
| **DetectLogicTests** | `RunDetectLogic_操作方法システムコード一致_操作不一致_非通知` | `DetectByOperationMethod` L493 False ＋ L502 `sysCodeMatch.Count > 0` True ＋ L505 `Any` False |
| **DetectLogicTests** | `RunDetectLogic_操作方法リスト該当なし_非通知` | `DetectByOperationMethod` L493 False ＋ L502 False（走 L508 `else`） |
| **DetectLogicTests** | `RunDetectLogic_操作カタログ対象外リソース_判定スキップ` | `DetectByOperationCatalog` L532 `if (blob \|\| file \|\| table)` 为 False |
| **DetectLogicTests** | `RunDetectLogic_操作カタログ未登録_エラーログ出力_非通知` | `DetectByOperationCatalog` L552 `if (listCount > 0)` 为 False（走 L576 `else`） |
| **DetectLogicTests** | `RunDetectLogic_操作カタログ不一致_エラーログ出力_非通知` | `DetectByOperationCatalog` L552 True ＋ L561 `if (!Union.Any)` True |
| **DetectLogicTests** | `RunDetectLogic_ホワイトリスト一致_非通知` | `DetectByOperationCatalog` L552 True ＋ L561 False ＋ L569 `if (whiteList.Any)` True |
| **DetectLogicTests** | `RunDetectLogic_ブラックリスト一致_通知対象` | `DetectByOperationCatalog` L552 True ＋ L561 False ＋ L569 False |
| **DetectLogicTests** | `RunDetectLogic_ストレージ検査対象Null_判定例外処理` | `CheckStorageAccount` L626 `if (IsNullOrEmpty(val) \|\| IsNullOrEmpty(chkVal))` True |
| **DetectLogicTests** | `RunDetectLogic_SQLクエリ空_判定スキップ` | `CheckSql` L649 `if (IsNullOrEmpty(sqlQuery) \|\| IsNullOrEmpty(chkVal))` True |
| **DetectLogicTests** | `RunDetectLogic_SQL除外テーブル一致_非通知` | `CheckSql` L666 `Any()` True ＋ L678 `if (!SqlQuery.Contains(chkVal))` False |
| **DetectLogicTests** | `RunDetectLogic_操作対象取得_全リソース種別確認` | `GetOperationTarget` L701–711 `switch` 全分支（blob/file/table/sqldb/sqlmi） |
| **GetDetectLogTests** | `Initialize_ログ一覧取得_正常終了` | L17 正常执行路径 |
| **GetDetectLogTests** | `Initialize_ログ一覧0件_正常終了` | L17 空集合边界 |
| **GetDetectLogTests** | `GetRenameLogList_ログリスト0件_空リスト返却` | L24 `.Where` 遍历 0 次 |
| **GetDetectLogTests** | `GetRenameLogList_xel以外のファイルあり_リネーム対象取得` | L24 `!= ".xel"` 为 True |
| **GetDetectLogTests** | `GetRenameLogList_xelファイルのみ_リネーム対象なし` | L24 `!= ".xel"` 全为 False |
| **GetDetectLogTests** | `GetRenameLogList_xelとその他混在_xel以外のみ抽出` | L24 `.Where` 多次遍历（True / False 混合） |
| **GetDetectLogTests** | `GetRenameLogList_ログリスト最大件数_正常終了` | L24 `.Where` 遍历最大次数边界 |
| **GetDetectLogTests** | `GetXelLogList_ログリスト0件_空リスト返却` | L36 `.Where` 遍历 0 次 |
| **GetDetectLogTests** | `GetXelLogList_xelファイル複数件_更新日時昇順ソート` | L36 `== ".xel"` True ＋ L37 `.OrderBy` 排序验证 |
| **GetDetectLogTests** | `GetXelLogList_xel以外のファイルのみ_空リスト返却` | L36 `== ".xel"` 全为 False |
| **GetDetectLogTests** | `GetXelLogList_xelとその他混在_xelのみ抽出` | L36 `.Where` 多次遍历（True / False 混合） |
| **GetDetectLogTests** | `GetXelLogList_ログリスト最大件数_正常終了` | L36 `.Where` / `.OrderBy` 最大次数边界 |
| **CosmosDBServiceTests** | `PostItem_DetectionHistoryModel_新規登録` | L24 `_container == null` True ＋ L26 `DetectionHistoryModel` True ＋ L36 `overwrite` False |
| **CosmosDBServiceTests** | `PostItem_UserIdManagementModel_上書き更新` | L24 True ＋ L30 `UserIdManagementModel` True ＋ L36 `overwrite` True |
| **CosmosDBServiceTests** | `PostItem_対象外モデル型_コンテナ未設定例外` | L24 True ＋ L26 False ＋ L30 False（触发 `NullReferenceException` 进 L47 `catch`） |
| **CosmosDBServiceTests** | `PostItem_コンテナ設定済み_再取得スキップ` | L24 `if (_container == null)` 为 False |
| **CosmosDBServiceTests** | `PostItem_DB登録例外_ログ出力_再スロー` | L47 `catch (Exception e)` |
| **CosmosDBServiceTests** | `GetAllItemList_OperationMethodModel_全件取得` | L62 `_container == null` True ＋ L64 `OperationMethodModel` True |
| **CosmosDBServiceTests** | `GetAllItemList_NotificationConditionsModel_全件取得` | L62 True ＋ L68 `NotificationConditionsModel` True |
| **CosmosDBServiceTests** | `GetAllItemList_OperationCatalogModel_全件取得` | L62 True ＋ L72 `OperationCatalogModel` True |
| **CosmosDBServiceTests** | `GetAllItemList_UserIdManagementModel_全件取得` | L62 True ＋ L76 `UserIdManagementModel` True |
| **CosmosDBServiceTests** | `GetAllItemList_対象外モデル型_コンテナ未設定例外` | L62 True ＋ L64~76 全 False（触发 `NullReferenceException` 进 L101 `catch`） |
| **CosmosDBServiceTests** | `GetAllItemList_コンテナ設定済み_再取得スキップ` | L62 `if (_container == null)` 为 False |
| **CosmosDBServiceTests** | `GetAllItemList_データ0件_空リスト返却` | L90 `while (HasMoreResults)` 循环 0 次 |
| **CosmosDBServiceTests** | `GetAllItemList_ページ内データ0件_空リスト返却` | L90 `while` 1 次 ＋ L95 `foreach` 循环 0 次 |
| **CosmosDBServiceTests** | `GetAllItemList_複数ページ最大件数_全件結合返却` | L90 `while` 多次 ＋ L95 `foreach` 多次 / 最大次数边界 |
| **CosmosDBServiceTests** | `GetAllItemList_DB取得例外_ログ出力_再スロー` | L101 `catch (Exception e)` |
| **DataLakeServiceTests** | `GetLogFileList_全コンテナ空_空リスト返却` | L107 `Parallel.ForEach` ＋ L114/L116 `foreach` 循环 0 次 |
| **DataLakeServiceTests** | `GetLogFileList_対象外拡張子のみ_空リスト返却` | L119 `if (Regex.IsMatch)` 为 False |
| **DataLakeServiceTests** | `GetLogFileList_複数コンテナ複数ファイル_全件取得` | L107 多次 ＋ L114/L116 多次 ＋ L119 `Regex.IsMatch` True |
| **DataLakeServiceTests** | `GetLogFileList_最大ファイル数_全件取得` | L107 / L114 / L116 循环最大次数边界 |
| **DataLakeServiceTests** | `GetLogFileList_取得例外_エラーログ出力_再スロー` | L140 `catch (Exception e)` |
| **DataLakeServiceTests** | `RenameLogFile_ファイルリスト0件_空リスト返却` | L158 `Parallel.ForEach` 循环 0 次 |
| **DataLakeServiceTests** | `RenameLogFile_未リネームファイル_リネーム実行` | L166 `if (!Regex.IsMatch)` 为 True |
| **DataLakeServiceTests** | `RenameLogFile_リネーム済みファイル_スキップ` | L166 `if (!Regex.IsMatch)` 为 False |
| **DataLakeServiceTests** | `RenameLogFile_複数ファイル混在_対象のみリネーム` | L158 `Parallel.ForEach` 多次（True / False 混合） |
| **DataLakeServiceTests** | `RenameLogFile_最大件数_全件処理` | L158 `Parallel.ForEach` 循环最大次数边界 |
| **DataLakeServiceTests** | `RenameLogFile_リネーム例外_エラーログ出力_再スロー` | L186 `catch (Exception e)` |
| **DataLakeServiceTests** | `GetLogContents_ストレージアカウント_正常取得` | L219 True ＋ L224 `while` 多次 ＋ L231 `if (storage)` True |
| **DataLakeServiceTests** | `GetLogContents_ストレージアカウント_UPN取得例外とResourceType判定` | `MappingModelStorageAccount` L389 `catch (NullReferenceException)` ＋ L396 `if (!Contains)` |
| **DataLakeServiceTests** | `GetLogContents_SQLMI_正常取得` | L219 True ＋ L231 False ＋ L236 `else if (sqlMI)` True |
| **DataLakeServiceTests** | `GetLogContents_CosmosDB_正常取得` | L219 True ＋ L231 False ＋ L236 False ＋ L241 `else if (cosmosDB)` True |
| **DataLakeServiceTests** | `GetLogContents_テキストログ0行_空リスト返却` | L219 True ＋ L224 `while (EndOfStream == false)` 循环 0 次 |
| **DataLakeServiceTests** | `GetLogContents_テキストログ最大行数_全件取得` | L224 `while (EndOfStream == false)` 循环最大次数边界 |
| **DataLakeServiceTests** | `GetLogContents_SQLDB_XELファイル正常取得` | L219 False ＋ L252 `else if (sqlDB)` True ＋ L263 `while` 多次 ＋ L270 `foreach` 多次 |
| **DataLakeServiceTests** | `GetLogContents_SQLDB_空XELファイル_空リスト返却` | L252 True ＋ L263 `while` 循环 0 次 ＋ L270 `foreach` 循环 0 次 |
| **DataLakeServiceTests** | `GetLogContents_対象外コンテナ_空リスト返却` | L219 False ＋ L252 False |
| **DataLakeServiceTests** | `GetLogContents_取得例外_一時ファイル削除_再スロー` | L278 `catch (Exception e)` ＋ L285 `finally` L288 `if (File.Exists)` True |
| **DataLakeServiceTests** | `GetLogContents_ストレージアカウント_UPNあり_Blob種別設定` | `MappingModelStorageAccount` L382 `else` ＋ L396 `if (Contains(blob))` True |
| **DataLakeServiceTests** | `GetLogContents_ストレージアカウント_UPN空文字_File種別設定` | `MappingModelStorageAccount` L379 `if (IsNullOrEmpty)` True ＋ L400 `else if (Contains(file))` True |
| **DataLakeServiceTests** | `GetLogContents_ストレージアカウント_UPN取得Null例外_Table種別設定` | `MappingModelStorageAccount` L389 `catch (NullReferenceException)` ＋ L404 `else if (Contains(table))` True |
| **DataLakeServiceTests** | `GetLogContents_ストレージアカウント_種別パス不一致_種別未設定` | `MappingModelStorageAccount` L396 / L400 / L404 全 False |
| **DataLakeServiceTests** | `GetLogContents_CosmosDB_ユーザーIDあり_正常マッピング` | `MappingModelCosmosDB` L460 `if (string.IsNullOrEmpty(UserId))` False |
| **DataLakeServiceTests** | `GetLogContents_CosmosDB_ユーザーID空_ハイフン設定` | `MappingModelCosmosDB` L460 `if (string.IsNullOrEmpty(UserId))` True |
| **DataLakeServiceTests** | `DeleteLogFile_通常ログ削除_正常終了` | L313 正常执行路径（未进入 L326） |
| **DataLakeServiceTests** | `DeleteLogFile_CosmosDBログ削除_フォルダ削除実行` | L326 `if (cosmosDBContainerNames.Contains)` 为 True |
| **DataLakeServiceTests** | `DeleteLogFile_通常ログ削除_コンフリクト発生_ログ出力なし` | L334 `catch (RequestFailedException e)` ＋ L337 `if (e.Status != Conflict)` False |
| **DataLakeServiceTests** | `DeleteLogFile_通常ログ削除_コンフリクト以外エラー_ログ出力のみ` | L334 `catch (RequestFailedException e)` ＋ L337 `if (e.Status != Conflict)` True |
| **DataLakeServiceTests** | `DeleteLogFile_CosmosDBフォルダ削除例外_ログ出力のみ` | L344 `catch (Exception e)` |
| **AlertServiceTests** | `Initialize_モジュール取得_正常終了` | L36–37 正常执行路径 |
| **AlertServiceTests** | `Alert_正常アラート送信_波括弧全角置換` | L53 字符串替换 ＋ L72 `_alert?.DoSetString` 非 null 分支 |
| **AlertServiceTests** | `Alert_モジュール未初期化_送信スキップ` | L72 `_alert?.DoSetString` 为 null 分支 |
| **AlertServiceTests** | `Alert_送信処理例外_エラーログ出力のみ` | L74 `catch (Exception ex)` |
| **AlertServiceTests** | `Alert_エラーメッセージのみ指定_デフォルトIDで送信` | L87 重载方法调用默认参数路径 |
| **APPLoggerTests** | `LogInitialize_ログ初期設定_正常終了` | L41–46 正常执行路径 |
| **APPLoggerTests** | `WriteLog_例外なし_波括弧全角置換でログ出力` | L86 `exception` null 分支（`?? string.Empty`）＋ L66 字符替换 |
| **APPLoggerTests** | `WriteLog_例外あり_例外情報付加でログ出力` | L86 `exception` 非 null 分支（`exception.ToString()`） |
| **APPLoggerTests** | `WriteLog_ログ出力例外発生_例外破棄` | L91 `catch` 块（静默吞掉异常） |
| **ErrorFileManagerTests** | `OutputErrorFile_ディレクトリとファイル未存在_新規作成` | L45 `if (!directory.Exists)` True ＋ L50 `if (!File.Exists)` True ＋ L142 `else` |
| **ErrorFileManagerTests** | `OutputErrorFile_既存ファイルあり_閾値未満_回数加算のみ` | L45 False ＋ L50 False（走 L59 `else`）＋ L67 `if (errorCount % Threshold == 0)` False |
| **ErrorFileManagerTests** | `OutputErrorFile_既存ファイルあり_閾値到達_緊急アラート発報` | L59 `else` ＋ L67 `if (errorCount % Threshold == 0)` True |
| **ErrorFileManagerTests** | `OutputErrorFile_SQLDBログ_専用ファイル名生成` | `GetErrorFileName` L121 `if (containerName == _sqldbContainerName)` True |
| **ErrorFileManagerTests** | `OutputErrorFile_SQLMIログ_専用ファイル名生成` | `GetErrorFileName` L128 `else if (containerName == _sqlmiContainerName)` True |
| **ErrorFileManagerTests** | `OutputErrorFile_CosmosDBログ_専用ファイル名生成` | `GetErrorFileName` L135 `else if (containerName == _cosmosDBContainerName)` True |
| **ErrorFileManagerTests** | `GetErrorFile_エラーファイル存在_内容取得` | L83–86 正常读取与反序列化 |
| **ErrorFileManagerTests** | `DeleteErrorFile_対象ファイル0件_正常終了` | L96 `foreach (var file in files)` 循环 0 次 |
| **ErrorFileManagerTests** | `DeleteErrorFile_対象ファイル1件_削除実行` | L96 `foreach (var file in files)` 循环 1 次 |
| **ErrorFileManagerTests** | `DeleteErrorFile_対象ファイル複数件_全件削除` | L96 `foreach (var file in files)` 循环多次 / 最大边界 |
| **ErrorFileManagerTests** | `IsExists_エラーファイルあり_True返却` | L107 `Length != 0` 为 True |
| **ErrorFileManagerTests** | `IsExists_エラーファイルなし_False返却` | L107 `Length != 0` 为 False |
| **UserAccountManagerTests** | `GetDBData_データあり_初期化完了` | L41–45 正常赋值路径 |
| **UserAccountManagerTests** | `GetDBData_データ0件_空リスト初期化` | L41–45 空集合边界 |
| **UserAccountManagerTests** | `GetUPN_DBにデータあり_履歴未登録_DBのUPN返却と履歴追加` | L62 `if (item == null)` False（走 L89 `else`）＋ L94 `if (!OldHistory.Contains)` True |
| **UserAccountManagerTests** | `GetUPN_DBにデータあり_履歴登録済み_重複追加なし` | L89 `else` ＋ L94 `if (!OldHistory.Contains)` False |
| **UserAccountManagerTests** | `GetUPN_DB未登録_Graphユーザー取得成功_新規リスト追加` | L62 True ＋ L68 False（走 L73 `else`）＋ L169 `TryGetValue("userPrincipalName")` True |
| **UserAccountManagerTests** | `GetUPN_DB未登録_Graphサービスプリンシパル取得成功` | L62 True ＋ L73 `else` ＋ L175 `catch (ODataError)` ＋ L186 `TryGetValue("displayName")` True |
| **UserAccountManagerTests** | `GetUPN_DB未登録_Graphサービスプリンシパルキーなし_ハイフン返却` | L62 True ＋ L68 True ＋ L175 `catch (ODataError)` ＋ L186 `TryGetValue("displayName")` False |
| **UserAccountManagerTests** | `GetUPN_DB未登録_Graphユーザーキーなし_ハイフン返却` | L62 True ＋ L68 True ＋ L169 `TryGetValue("userPrincipalName")` False |
| **UserAccountManagerTests** | `GetUPN_GraphAPI通信例外_ログ出力とハイフン返却` | L62 True ＋ L68 True ＋ L194 `catch (Exception ex)` |
| **UserAccountManagerTests** | `GetUPN_サービスプリンシパル例外_ハイフン返却` | L62 True ＋ L68 True ＋ L175 `catch (ODataError)` ＋ L189 `catch (Exception)` |
| **UserAccountManagerTests** | `GetUPN_GraphClient未初期化_証明書未検出例外` | `GetGraphClient` L209 True ＋ `GetCert` L250 `if (cert is null)` True |
| **UserAccountManagerTests** | `GetUPN_GraphClient未初期化_証明書取得成功で生成` | `GetGraphClient` L209 True ＋ `GetCert` L250 False ＋ `CreateGraphClient` L264–285 |
| **UserAccountManagerTests** | `RegisterDataInDB_追加更新ともに0件_登録スキップ` | L116 `foreach (registerData)` 0 次 ＋ L132 `foreach (OldHistory)` 0 次 |
| **UserAccountManagerTests** | `RegisterDataInDB_新規追加データあり_ハイフン除外で登録` | L116 `foreach (registerData)` 多次（含 `Upn != "-"` 过滤） |
| **UserAccountManagerTests** | `RegisterDataInDB_新規追加例外_例外破棄で処理継続` | L122 `catch` 块（吞掉异常继续循环） |
| **UserAccountManagerTests** | `RegisterDataInDB_更新対象あり_上書き登録実行` | L132 `foreach (OldHistory)` 多次 ＋ L135 `if (item is not null)` True |
| **UserAccountManagerTests** | `RegisterDataInDB_更新対象DB未存在_スキップ` | L135 `if (item is not null)` False |
| **UserAccountManagerTests** | `RegisterDataInDB_更新登録例外_例外破棄で処理継続` | L142 `catch` 块（吞掉异常继续循环） |
| **UserAccountManagerTests** | `RegisterDataInDB_最大件数登録更新_全件正常終了` | L116 / L132 循环最大次数边界 |
| **XelFileManagerTests** | `OutputFileAsync_ディレクトリ未存在_新規作成してファイル出力` | L19 `if (!directory.Exists)` 为 True |
| **XelFileManagerTests** | `OutputFileAsync_ディレクトリ既存_ファイル上書き出力` | L19 `if (!directory.Exists)` 为 False |
| **XelFileManagerTests** | `GetFileAsync_管理ファイル存在_内容取得` | L35–37 正常读取与反序列化 |
| **XelFileManagerTests** | `DeleteFileAsync_対象ファイル0件_正常終了` | L47 `foreach (var file in files)` 循环 0 次 |
| **XelFileManagerTests** | `DeleteFileAsync_対象ファイル1件_削除実行` | L47 `foreach (var file in files)` 循环 1 次 |
| **XelFileManagerTests** | `DeleteFileAsync_対象ファイル複数件_全件削除` | L47 `foreach (var file in files)` 循环多次 / 最大边界 |
| **XelFileManagerTests** | `IsExists_管理ファイルあり_True返却` | L58 `Length != 0` 为 True |
| **XelFileManagerTests** | `IsExists_管理ファイルなし_False返却` | L58 `Length != 0` 为 False |
| **NotificationFuncTests** | `Run_正常処理_削除フラグ除外で検知実行` | L107/L108 `.Where` 过滤 ＋ L117 `Parallel.ForEach` 多次 ＋ L119 `if (!IsCancellationRequested)` True |
| **NotificationFuncTests** | `Run_リネームログ0件_検知ロジックスキップ` | L117 `Parallel.ForEach` 循环 0 次 |
| **NotificationFuncTests** | `Run_リネームログ最大件数_並列制限内で正常終了` | L117 `Parallel.ForEach` 循环最大次数边界（`MaxDegreeOfParallelism = 10`） |
| **NotificationFuncTests** | `Run_キャンセルトークン要求あり_検知ロジックスキップ` | L119 `if (!cancellationToken.IsCancellationRequested)` 为 False |
| **NotificationFuncTests** | `Run_処理中例外発生_キャッチして正常終了` | L130 `catch (Exception)` |
| **NotificationFuncTests** | `RunHealthCheck_ヘルスチェック実行_SUCCESS返却` | L148 正常返回路径 |