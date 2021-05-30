<?php


require_once __DIR__ . '/database.class.php';

create_table_users();

exit( 0 );

// --------------------------
function has_table( $tblname )
{
	$db = DB::getConnection();
	
	try
	{
		$st = $db->query( 'SELECT DATABASE()' );
		$dbname = $st->fetch()[0];

		$st = $db->prepare( 
			'SELECT * FROM information_schema.tables WHERE table_schema = :dbname AND table_name = :tblname LIMIT 1' );
		$st->execute( ['dbname' => $dbname, 'tblname' => $tblname] );
		if( $st->rowCount() > 0 )
			return true;
	}
	catch( PDOException $e ) { exit( "PDO error [show tables]: " . $e->getMessage() ); }

	return false;
}


function create_table_users()
{
	$db = DB::getConnection();

	if( has_table( 'ricochet_robots_players' ) )
		exit( 'Table ricochet_robots_players already exists.' );

	try
	{
		$st = $db->prepare( 
			'CREATE TABLE IF NOT EXISTS ricochet_robots_players (' .
			'id int NOT NULL PRIMARY KEY AUTO_INCREMENT,' .
			'username varchar(50) NOT NULL,' .
			'password_hash varchar(255) NOT NULL,'.
			'email varchar(50) NOT NULL,' .
			'registration_sequence varchar(20) NOT NULL,' .
			'has_registered int,' .
            'games_played int,' .
            'tokens_won int, '.
            'games_won int)'
		);

		$st->execute();
	}
	catch( PDOException $e ) { exit( "PDO error [create dz2_users]: " . $e->getMessage() ); }

	echo "Created table ricochet_robots_players.<br />";
}

?> 