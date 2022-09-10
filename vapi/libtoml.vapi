[CCode(cheader_filename = "toml.h")]
namespace Toml {
	[Compact]
	[CCode(cname = "toml_table_t", cprefix = "toml_", free_function = "", has_type_id = false)]
	public class Table {
		/** Parse a string containing the full config.
		 * Return a table on success, or 0 otherwise.
		 * Caller must toml_free(the-return-value) after use.
		 */
		[CCode(cname = "toml_parse")]
		public static Table? from_string(string? config, out uint8[] error);

		/** Parse a file. Return a table on success, or 0 otherwise.
		 * Caller must toml_free(the-return-value) after use.
		 */
		[CCode(cname = "toml_parse_file")]
		public static Table? from_file(GLib.FileStream? file, out uint8[] error);

		/** ... retrieve the key in table at keyidx. Return 0 if out of range. */
		[CCode(cname = "toml_key_in")]
		public unowned string? get_key(int index);

		/** ... returns 1 if key exists in tab, 0 otherwise */
		[CCode(cname = "toml_key_exists")]
		public unowned bool contains(string key);

		/** Return the number of key-values in a table */
		public int size {
			[CCode(cname = "toml_table_nkval")] get;
		}

		/** Return the number of arrays in a table */
		public int array_count {
			[CCode(cname = "toml_table_narr")] get;
		}

		/** Return the number of sub-tables in a table */
		public int table_count {
			[CCode(cname = "toml_table_ntab")] get;
		}

		/** Return the key of a table */
		public string key {
			[CCode(cname = "toml_table_key")] get;
		}

		public string? get_string(string key) {
			var datum = string_in(key);

			if (datum.ok) return datum.string;

			return null;
		}

		public bool? get_bool(string key) {
			var datum = bool_in(key);

			if (datum.ok) return datum.bool;

			return null;
		}

		public int64? get_int(string key) {
			var datum = int_in(key);

			if (datum.ok) return datum.int64;

			return null;
		}

		public double? get_double(string key) {
			var datum = double_in(key);

			if (datum.ok) return datum.double;

			return null;
		}

		public Timestamp? get_timestamp(string key) {
			var datum = timestamp_in(key);

			if (datum.ok) return datum.timestamp;

			return null;
		}

		[CCode(cname = "toml_array_in")]
		public Array? get_array(string key);

		[CCode(cname = "toml_table_in")]
		public Table? get_table(string key);

		/** Free the table returned by toml_parse() or toml_parse_file(). Once
		 * this function is called, any handles accessed through this tab
		 * directly or indirectly are no longer valid.
		 */
		[DestroysInstance]
		public void free();

		/* ... retrieve values using key. */
		private Datum string_in(string key);
		private Datum bool_in(string key);
		private Datum int_in(string key);
		private Datum double_in(string key);
		private Datum timestamp_in(string key);
	}

	[Compact]
	[CCode(cname = "toml_array_t", cprefix = "toml_", free_function = "", has_type_id = false)]
	public class Array {
		public int size {
			[CCode(cname = "toml_array_nelem")] get;
		}

		/** Return the array kind: 't'able, 'a'rray, 'v'alue, 'm'ixed */
		public char kind {
			[CCode(cname = "toml_array_kind")] get;
		}

		/** For array kind 'v'alue, return the type of values
		    i:int, d:double, b:bool, s:string, t:time, D:date, T:timestamp, 'm'ixed
		    0 if unknown
		 */
		public char? type {
			[CCode(cname = "toml_array_type")] get;
		}

		/** Return the key of an array */
		public string key {
			[CCode(cname = "toml_array_key")] get;
		}

		public string? get_string(int index) {
			var datum = string_at(index);

			if (datum.ok) return datum.string;

			return null;
		}

		public bool? get_bool(int index) {
			var datum = bool_at(index);

			if (datum.ok) return datum.bool;

			return null;
		}

		public int64? get_int(int index) {
			var datum = int_at(index);

			if (datum.ok) return datum.int64;

			return null;
		}

		public double? get_double(int index) {
			var datum = double_at(index);

			if (datum.ok) return datum.double;

			return null;
		}

		public Timestamp? get_timestamp(int index) {
			var datum = timestamp_at(index);

			if (datum.ok) return datum.timestamp;

			return null;
		}

		[CCode(cname = "toml_array_at")]
		public Array? get_array(int index);

		[CCode(cname = "toml_table_at")]
		public Table? get_table(int index);


		/* ... retrieve values using index. */
		private Datum string_at(int index);
		private Datum bool_at(int index);
		private Datum int_at(int index);
		private Datum double_at(int index);
		private Datum timestamp_at(int index);
	}

	/** Timestamp types. The year, month, day, hour, minute, second, z
	 * fields may be NULL if they are not relevant. e.g. In a DATE
	 * type, the hour, minute, second and z fields will be NULLs.
	 */
	[CCode(cname = "toml_timestamp_t", destroy_function = "", has_type_id = false)]
	public struct Timestamp {
		[CCode(cname = "__buffer.year")]
		private int buffer_year;

		[CCode(cname = "__buffer.month")]
		private int buffer_month;

		[CCode(cname = "__buffer.day")]
		private int buffer_day;

		[CCode(cname = "__buffer.hour")]
		private int buffer_hour;

		[CCode(cname = "__buffer.minute")]
		private int buffer_minute;

		[CCode(cname = "__buffer.second")]
		private int buffer_second;

		[CCode(cname = "__buffer.millisec")]
		private int buffer_millisec;

		[CCode(cname = "__buffer.z")]
		private char buffer_z[10];

		int? year;
		int? month;
		int? day;
		int? hour;
		int? minute;
		int? second;
		int? millisec;
		string? z;
	}

	[SimpleType]
	[CCode(cname = "toml_datum_t", destroy_function = "", has_type_id = false)]
	public struct Datum {
		bool ok;

		[CCode(cname = "u.ts")]
		Timestamp? timestamp;

		[CCode(cname = "u.s")]
		string? string;

		[CCode(cname = "u.b")]
		bool bool;

		[CCode(cname = "u.i")]
		int64 int64;

		[CCode(cname = "u.d")]
		double double;
	}
}
