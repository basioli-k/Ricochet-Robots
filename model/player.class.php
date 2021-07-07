<?php

class Player extends Model{

	protected static $table = "ricochet_robots_players";
	protected static $attributes = array("id" => "int", "username" => "string", "password_hash" => "string", "email" => "string",
                                        "registration_sequence" => "string", "has_registered" => "int", "games_played" => "int",
										"tokens_won" => "int", "games_won" => "int");

	public $scored_targets; //array gdje ubacujemo sve tokene koje igrac osvoji

	//array of steps the player offered for his solution
	public $offered_solution;

	function __construct($row){
		$this->columns = $row;
	}
	
	function move_robot(&$board, $robot, $direction){
		if($direction === "left"){
			$i_shift = 0;
			$j_shift = -1;
		}
		else if($direction === "top"){
			$i_shift = -1;
			$j_shift = 0;
		}
		else if($direction === "right"){
			$i_shift = 0;
			$j_shift = 1;
		}
		else if($direction === "bottom"){
			$i_shift = 1;
			$j_shift = 0;
		}
		else{
			echo "Invalid direction.";
			return 0;
		}

		$robot_i = NULL;
		$robot_j = NULL;
		for($i = 0 ; $i < count($board); ++$i)
			for($j = 0 ; $j < count($board[$i]); ++$j)
				if( $board[$i][$j]->robot === $robot){
					$robot_i = $i;
					$robot_j = $j;
				}

		if( $robot_i === NULL || $robot_j === NULL){
			echo "This robot doesn't exist";
			return 0;
		}

		$i = $robot_i;
		$j = $robot_j;
		
		$steps = 0;
		
		//todo dodati checkove da i+ i_shift ne izlazi iz ploce...
		//moguce da ne treba
		while( !$board[$i][$j]->walls[$direction] && $board[$i + $i_shift][$j + $j_shift]->robot === NULL) {
				$i += $i_shift;
				$j += $j_shift;
				$steps += 1;
		}

		$board[$robot_i][$robot_j]->robot = NULL;
		$board[$i][$j]->robot = $robot;

		return $steps; 
	}

	static function getPlayer($username) {
		$players = Player::where(array("username" => $username));
		
		if(!$players || !isset($players[0]))
			return false;
			
		return $players[0];
	}
};
?>