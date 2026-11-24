<?php

class Player extends Model{

	protected static $table = "ricochet_robots_players";
	protected static $attributes = array("id" => "int", "username" => "string", "password_hash" => "string", "email" => "string",
                                        "registration_sequence" => "string", "has_registered" => "int", "games_played" => "int",
										"tokens_won" => "int", "games_won" => "int");

	public $scored_targets; //broj tokena na kraju

	function __construct($row){
		$this->columns = $row;
	}

	static function getPlayer($username, $email = null) {
		$args = array();
		if($username != null)
			$args["username"] = $username;
			
		if($email != null)
			$args["email"] = $email;

		$players = Player::where($args, "OR");
		
		if(!$players || !isset($players[0]))
			return false;
			
		return $players[0];
	}
};
?>