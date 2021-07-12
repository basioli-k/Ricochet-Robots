<?php

// Popunjavamo tablice u bazi "probnim" podacima.
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

		$st->execute( array( 'username' => 'avirovic', 'password' => password_hash( 'googleboy42', PASSWORD_DEFAULT ), "email" => "ivan.avirovic@gmail.com" ) );
		$st->execute( array( 'username' => 'basioli', 'password' => password_hash( 'karlovasifra', PASSWORD_DEFAULT ), "email" => "k.basioli@gmail.com") );
		$st->execute( array( 'username' => 'duric', 'password' => password_hash( 'zupanja032', PASSWORD_DEFAULT ), "email" => "duricant.math@pmf.hr" ) );
		$st->execute( array( 'username' => 'igrac', 'password' => password_hash( 'igrac', PASSWORD_DEFAULT ), "email" => "fake@mail.hr" ) );
	}
	catch( PDOException $e ) { exit( "PDO error [insert dz2_users]: " . $e->getMessage() ); }

	echo "Ubacio u tablicu dz2_users.<br />";
}


?> 
 
 