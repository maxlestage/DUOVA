import CSQLite
import Foundation

/// An error of SQLite: its result code and its message.
public struct SQLiteError: Error, Equatable, CustomStringConvertible {
    public let code: Int32
    public let message: String

    public var description: String { "SQLite error \(code): \(message)" }
}

/// A value bound to, or read from, an SQL statement.
public enum SQLValue: Equatable, Sendable {
    case integer(Int64)
    case real(Double)
    case text(String)
    case null
}

/// A row returned by `Database.query`.
public struct SQLRow: Sendable {
    public let values: [SQLValue]

    public func int(_ column: Int) -> Int {
        switch values[column] {
        case .integer(let value): Int(value)
        case .real(let value): Int(value)
        default: 0
        }
    }

    public func double(_ column: Int) -> Double {
        switch values[column] {
        case .integer(let value): Double(value)
        case .real(let value): value
        default: 0
        }
    }

    public func text(_ column: Int) -> String {
        if case .text(let value) = values[column] { value } else { "" }
    }
}

/// One connection to an SQLite database: a file, or `":memory:"`. Prepared statements
/// are kept and reused. Not shared between threads (SQLite is built in multi-thread mode).
public final class Database {
    private var handle: OpaquePointer?
    private var statements: [String: OpaquePointer] = [:]

    /// The version of the SQLite library compiled into the app.
    public static var sqliteVersion: String { String(cString: sqlite3_libversion()) }

    public init(path: String) throws {
        let flags = SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE | SQLITE_OPEN_NOMUTEX | SQLITE_OPEN_EXRESCODE
        let code = sqlite3_open_v2(path, &handle, flags, nil)
        guard code == SQLITE_OK else {
            let error = SQLiteError(code: code, message: handle.map { String(cString: sqlite3_errmsg($0)) } ?? "open")
            sqlite3_close_v2(handle)
            handle = nil
            throw error
        }
        sqlite3_busy_timeout(handle, 2_000)
    }

    deinit {
        for statement in statements.values {
            sqlite3_finalize(statement)
        }
        sqlite3_close_v2(handle)
    }

    /// Runs one or more statements without parameters (the schema, pragmas).
    public func execute(_ sql: String) throws {
        try check(sqlite3_exec(handle, sql, nil, nil, nil))
    }

    /// Runs a statement with its parameters (`?1`, `?2`… or `?`).
    public func run(_ sql: String, _ values: [SQLValue] = []) throws {
        let statement = try prepare(sql, values)
        defer { sqlite3_reset(statement) }
        var code = sqlite3_step(statement)
        while code == SQLITE_ROW { code = sqlite3_step(statement) }
        try check(code, done: SQLITE_DONE)
    }

    /// Every row of a query.
    public func query(_ sql: String, _ values: [SQLValue] = []) throws -> [SQLRow] {
        let statement = try prepare(sql, values)
        defer { sqlite3_reset(statement) }
        var rows: [SQLRow] = []
        let columns = sqlite3_column_count(statement)
        while true {
            let code = sqlite3_step(statement)
            if code == SQLITE_DONE { break }
            guard code == SQLITE_ROW else { try check(code); break }
            rows.append(SQLRow(values: (0..<columns).map { column(statement, $0) }))
        }
        return rows
    }

    /// Runs `body` in a transaction: everything is written, or nothing. Inside another
    /// transaction, it is a savepoint of it.
    public func transaction<T>(_ body: () throws -> T) throws -> T {
        let nested = sqlite3_get_autocommit(handle) == 0
        try execute(nested ? "SAVEPOINT nested" : "BEGIN IMMEDIATE")
        do {
            let result = try body()
            try execute(nested ? "RELEASE nested" : "COMMIT")
            return result
        } catch {
            try? execute(nested ? "ROLLBACK TO nested; RELEASE nested" : "ROLLBACK")
            throw error
        }
    }

    /// `PRAGMA user_version`: the version of the schema.
    public var userVersion: Int {
        get throws { try query("PRAGMA user_version").first?.int(0) ?? 0 }
    }

    public func setUserVersion(_ version: Int) throws {
        try execute("PRAGMA user_version = \(version)")
    }

    /// The number of rows changed by the last statement.
    public var changes: Int { Int(sqlite3_changes(handle)) }

    // MARK: Statements

    private func prepare(_ sql: String, _ values: [SQLValue]) throws -> OpaquePointer {
        let statement: OpaquePointer
        if let cached = statements[sql] {
            statement = cached
        } else {
            var prepared: OpaquePointer?
            try check(sqlite3_prepare_v3(handle, sql, -1, UInt32(SQLITE_PREPARE_PERSISTENT), &prepared, nil))
            guard let prepared else { throw SQLiteError(code: SQLITE_MISUSE, message: "empty statement: \(sql)") }
            statements[sql] = prepared
            statement = prepared
        }
        sqlite3_clear_bindings(statement)
        for (offset, value) in values.enumerated() {
            let index = Int32(offset + 1)
            let code: Int32 =
                switch value {
                case .integer(let int): sqlite3_bind_int64(statement, index, int)
                case .real(let double): sqlite3_bind_double(statement, index, double)
                case .text(let text): sqlite3_bind_text(statement, index, text, -1, transient)
                case .null: sqlite3_bind_null(statement, index)
                }
            try check(code)
        }
        return statement
    }

    private func column(_ statement: OpaquePointer, _ index: Int32) -> SQLValue {
        switch sqlite3_column_type(statement, index) {
        case SQLITE_INTEGER: .integer(sqlite3_column_int64(statement, index))
        case SQLITE_FLOAT: .real(sqlite3_column_double(statement, index))
        case SQLITE_NULL: .null
        default: .text(sqlite3_column_text(statement, index).map { String(cString: $0) } ?? "")
        }
    }

    private func check(_ code: Int32, done: Int32 = SQLITE_OK) throws {
        guard code != done && code != SQLITE_OK else { return }
        throw SQLiteError(code: code, message: String(cString: sqlite3_errmsg(handle)))
    }

    /// `SQLITE_TRANSIENT`: SQLite copies the text before the call returns.
    private var transient: sqlite3_destructor_type {
        unsafeBitCast(-1, to: sqlite3_destructor_type.self)
    }
}

extension SQLValue {
    init(_ uuid: UUID) { self = .text(uuid.uuidString) }
    init(_ date: Date) { self = .real(date.timeIntervalSince1970) }
    init(_ int: Int) { self = .integer(Int64(int)) }
}
