<?php

/**
 * PDO singleton. Credentials come from the environment (RR_DB_DSN,
 * RR_DB_USER, RR_DB_PASSWORD) so they are never committed.
 */
class DB
{
	private static $db = null;

	private function __construct() { }
	private function __clone() { }

	public static function getConnection()
	{
		if( DB::$db === null )
		{
			$dsn = getenv( 'RR_DB_DSN' );
			if( $dsn === false )
				exit( 'Database is not configured: set RR_DB_DSN, RR_DB_USER and RR_DB_PASSWORD.' );

			try
			{
				DB::$db = new PDO( $dsn, getenv( 'RR_DB_USER' ) ?: '', getenv( 'RR_DB_PASSWORD' ) ?: '' );
				DB::$db->setAttribute( PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION );
			}
			catch( PDOException $e ) { exit( 'Database connection failed.' ); }
		}
		return DB::$db;
	}
}

?>
