<?php
require_once("constants.php");
require_once("field.php");

class Player{
	public $name;

	public $scored_targets; //array gdje ubacujemo sve tokene koje igrac osvoji

	//array of steps the player offered for his solution
	public $offered_solution;

	function __construct($name){
		$this->name = $name;
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
		
		while( !$board[$i][$j]->walls[$direction] &&  ($board[$i][$j]->robot === NULL || $board[$i][$j]->robot === $robot) ){
				$i += $i_shift;
				$j += $j_shift;
				$steps += 1;
		}

		$board[$robot_i][$robot_j]->robot = NULL;
		$board[$i][$j]->robot = $robot;

		return $steps; 
	}
};
?>