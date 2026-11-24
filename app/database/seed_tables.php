<?php

// Popunjavamo tablice u bazi "probnim" podacima.
// CLI only: this file sits under the web root and prints passwords.
if( PHP_SAPI !== 'cli' )
	exit( 1 );

require_once __DIR__ . '/database.class.php';

seed_table_users();

exit( 0 );

// ------------------------------------------
function seed_table_users()
{
	$db = DB::getConnection();

	// Ubaci neke korisnike unutra
	try
	{
		$st = $db->prepare( 'INSERT INTO ricochet_robots_players(username, password_hash, email, registration_sequence, has_registered, ' . 
                            'games_played, tokens_won, games_won) VALUES (:username, :password, :email, \'abc\', \'1\', \'0\', \'0\', \'0\')' );

		// Demo accounts only. Passwords are random and printed once; never seed real people's data.
		foreach( array( 'alice', 'bob', 'igrac' ) as $username )
		{
			$password = bin2hex( random_bytes( 8 ) );
			$st->execute( array( 'username' => $username, 'password' => password_hash( $password, PASSWORD_DEFAULT ),
				"email" => $username . "@example.invalid" ) );
			echo $username . ': ' . $password . '<br />';
		}
	}
	catch( PDOException $e ) { exit( "PDO error [insert dz2_users]: " . $e->getMessage() ); }

	echo "Ubacio u tablicu dz2_users.<br />";
}


?> 
 
 